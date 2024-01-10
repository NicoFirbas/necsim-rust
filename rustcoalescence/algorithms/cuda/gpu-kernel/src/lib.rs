#![deny(clippy::pedantic)]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(decl_macro)]
#![feature(c_str_literals)]
#![cfg_attr(target_os = "cuda", feature(abi_ptx))]
#![cfg_attr(target_os = "cuda", feature(asm_experimental_arch))]
#![cfg_attr(target_os = "cuda", feature(alloc_error_handler))]
#![allow(long_running_const_eval)]
#![recursion_limit = "1024"]

extern crate alloc;

#[cfg(target_os = "cuda")]
use core::ops::ControlFlow;

// FIXME: why pub use?
pub use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

// FIXME: why pub use?
pub use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::{MinSpeciationTrackingEventSampler, SpeciationSample},
};

// FIXME: why pub use?
pub use rust_cuda::lend::RustToCuda;

#[rust_cuda::kernel::kernel(pub use link! for impl)]
#[kernel(
    allow(ptx::double_precision_use),
    forbid(ptx::local_memory_usage, ptx::register_spills)
)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn simulate<
    M: MathsCore + Sync,
    H: Habitat<M> + RustToCuda + Sync,
    G: PrimeableRng<M> + RustToCuda + Sync,
    S: LineageStore<M, H> + RustToCuda + Sync,
    X: EmigrationExit<M, H, G, S> + RustToCuda + Sync,
    D: DispersalSampler<M, H, G> + RustToCuda + Sync,
    C: CoalescenceSampler<M, H, S> + RustToCuda + Sync,
    T: TurnoverRate<M, H> + RustToCuda + Sync,
    N: SpeciationProbability<M, H> + RustToCuda + Sync,
    E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda + Sync,
    I: ImmigrationEntry<M> + RustToCuda + Sync,
    A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda + Sync,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
>(
    simulation: &rust_cuda::kernel::param::PtxJit<
        rust_cuda::kernel::param::DeepPerThreadBorrow<
            necsim_core::simulation::Simulation<M, H, G, S, X, D, C, T, N, E, I, A>,
        >,
    >,
    task_list: &mut rust_cuda::kernel::param::PtxJit<
        rust_cuda::kernel::param::DeepPerThreadBorrow<
            necsim_impls_cuda::value_buffer::ValueBuffer<necsim_core::lineage::Lineage, true, true>,
        >,
    >,
    event_buffer_reporter: &mut rust_cuda::kernel::param::PtxJit<
        rust_cuda::kernel::param::DeepPerThreadBorrow<
            necsim_impls_cuda::event_buffer::EventBuffer<ReportSpeciation, ReportDispersal>,
        >,
    >,
    min_spec_sample_buffer: &mut rust_cuda::kernel::param::PtxJit<
        rust_cuda::kernel::param::DeepPerThreadBorrow<
            necsim_impls_cuda::value_buffer::ValueBuffer<SpeciationSample, false, true>,
        >,
    >,
    next_event_time_buffer: &mut rust_cuda::kernel::param::PtxJit<
        rust_cuda::kernel::param::DeepPerThreadBorrow<
            necsim_impls_cuda::value_buffer::ValueBuffer<
                necsim_core_bond::PositiveF64,
                false,
                true,
            >,
        >,
    >,
    total_time_max: &rust_cuda::kernel::param::ShallowInteriorMutable<
        core::sync::atomic::AtomicU64,
    >,
    total_steps_sum: &rust_cuda::kernel::param::ShallowInteriorMutable<
        core::sync::atomic::AtomicU64,
    >,
    max_steps: rust_cuda::kernel::param::PerThreadShallowCopy<u64>,
    max_next_event_time: rust_cuda::kernel::param::PerThreadShallowCopy<
        necsim_core_bond::NonNegativeF64,
    >,
) {
    // TODO: use simulation with non-allocating clone
    let mut simulation = unsafe { core::mem::ManuallyDrop::new(core::ptr::read(simulation)) };

    task_list.with_value_for_core(|task| {
        // Discard the prior task (the simulation is just a temporary local copy)
        core::mem::drop(
            simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(task),
        );

        // Discard the prior sample (the simulation is just a temporary local copy)
        simulation.event_sampler_mut().replace_min_speciation(None);

        let mut final_next_event_time = None;

        let (time, steps) = simulation.simulate_incremental_early_stop(
            |_, steps, next_event_time| {
                final_next_event_time = Some(next_event_time);

                if steps >= max_steps || next_event_time >= max_next_event_time {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
            event_buffer_reporter,
        );

        next_event_time_buffer.put_value_for_core(final_next_event_time);

        if steps > 0 {
            total_time_max.fetch_max(time.get().to_bits(), core::sync::atomic::Ordering::Relaxed);
            total_steps_sum.fetch_add(steps, core::sync::atomic::Ordering::Relaxed);
        }

        min_spec_sample_buffer
            .put_value_for_core(simulation.event_sampler_mut().replace_min_speciation(None));

        simulation
            .active_lineage_sampler_mut()
            .replace_active_lineage(None)
    });
}

#[cfg(target_os = "cuda")]
mod cuda_prelude {
    use rust_cuda::device::alloc::PTXAllocator;

    #[global_allocator]
    static _GLOBAL_ALLOCATOR: PTXAllocator = PTXAllocator;

    #[cfg(not(debug_assertions))]
    #[panic_handler]
    fn panic(_panic_info: &::core::panic::PanicInfo) -> ! {
        rust_cuda::device::utils::exit()
    }

    #[cfg(debug_assertions)]
    #[panic_handler]
    fn panic(info: &::core::panic::PanicInfo) -> ! {
        rust_cuda::device::utils::pretty_panic_handler(info, true, true)
    }

    #[cfg(not(debug_assertions))]
    #[alloc_error_handler]
    fn alloc_error_handler(_: core::alloc::Layout) -> ! {
        rust_cuda::device::utils::exit()
    }

    #[cfg(debug_assertions)]
    #[alloc_error_handler]
    fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
        rust_cuda::device::utils::pretty_alloc_error_handler(layout)
    }
}
