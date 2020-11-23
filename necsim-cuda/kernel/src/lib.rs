#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(min_const_generics)]

extern crate alloc;

#[macro_use]
extern crate specialiser;

use rust_cuda::{
    device::{nvptx, utils},
    println,
};

#[global_allocator]
static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

#[panic_handler]
fn panic(panic_info: &::core::panic::PanicInfo) -> ! {
    println!(
        "Panic occurred at {:?}: {:?}!",
        panic_info.location(),
        panic_info
            .message()
            .unwrap_or(&format_args!("unknown reason"))
    );

    unsafe { nvptx::trap() }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    unsafe { nvptx::trap() }
}

struct F32(f32);
struct F64(f64);

impl core::fmt::Debug for F32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ryu::Buffer::new().format(self.0))
    }
}

impl core::fmt::Debug for F64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ryu::Buffer::new().format(self.0))
    }
}

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, HabitatToU64Injection,
        IncoherentLineageStore, LineageReference, PrimeableRng, SingularActiveLineageSampler,
    },
    simulation::Simulation,
};
use rust_cuda::{common::RustToCuda, device::BorrowFromRust};
use rustacuda_core::DeviceCopy;

use necsim_impls_cuda::{
    event_buffer::{common::EventBufferCudaRepresentation, device::EventBufferDevice},
    task_list::{common::TaskListCudaRepresentation, device::TaskListDevice},
};

use core::sync::atomic::{AtomicU64, Ordering};

extern "C" {
    static global_time_max: AtomicU64;
    static global_steps_sum: AtomicU64;
}

/// # Safety
/// This CUDA kernel is unsafe as it is called with raw c_void pointers
#[no_mangle]
pub unsafe extern "ptx-kernel" fn simulate(
    simulation_c_ptr: *mut core::ffi::c_void,
    task_list_c_ptr: *mut core::ffi::c_void,
    event_buffer_c_ptr: *mut core::ffi::c_void,
    max_steps: u64,
) {
    specialise!(simulate_generic)(
        simulation_c_ptr as *mut _,
        task_list_c_ptr as *mut _,
        event_buffer_c_ptr as *mut _,
        max_steps,
    )
}

unsafe fn simulate_generic<
    H: HabitatToU64Injection + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: EventSampler<H, G, D, R, S, C> + RustToCuda,
    A: SingularActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    simulation_ptr: *mut <Simulation<H, G, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
    task_list_ptr: *mut TaskListCudaRepresentation<H, R>,
    event_buffer_ptr: *mut EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
    max_steps: u64,
) {
    Simulation::with_borrow_from_rust_mut(simulation_ptr, |simulation| {
        TaskListDevice::with_borrow_from_rust_mut(task_list_ptr, |task_list| {
            task_list.with_task_for_core(|task| {
                let saved_task = simulation
                    .active_lineage_sampler_mut()
                    .replace_active_lineage(task);

                EventBufferDevice::with_borrow_from_rust_mut(
                    event_buffer_ptr,
                    |event_buffer_reporter| {
                        let (time, steps) =
                            simulation.simulate_incremental(max_steps, event_buffer_reporter);

                        if steps > 0 {
                            global_time_max.fetch_max(time.to_bits(), Ordering::Relaxed);
                            global_steps_sum.fetch_add(steps, Ordering::Relaxed);
                        }
                    },
                );

                simulation
                    .active_lineage_sampler_mut()
                    .replace_active_lineage(saved_task)
            })
        })
    })
}