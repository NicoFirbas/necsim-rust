#![deny(clippy::pedantic)]
#![feature(min_const_generics)]
#![feature(slice_fill)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate bitvec;

use anyhow::Result;

use rustacuda::{
    function::{BlockSize, GridSize},
    stream::{Stream, StreamFlags},
};

use necsim_core::{
    cogs::{
        DispersalSampler, HabitatToU64Injection, IncoherentLineageStore, LineageReference, RngCore,
    },
    simulation::Simulation,
};

use necsim_impls_cuda::{
    event_buffer::host::EventBufferHost, task_list::host::TaskListHost,
    value_buffer::host::ValueBufferHost,
};
use necsim_impls_no_std::reporter::ReporterContext;

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::poisson::PoissonEventTimeSampler,
        IndependentActiveLineageSampler as ActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler as CoalescenceSampler,
    event_sampler::independent::IndependentEventSampler as EventSampler,
    rng::wyhash::WyHash as Rng,
};

use necsim_impls_cuda::cogs::rng::CudaRng;

use rust_cuda::{common::RustToCuda, host::CudaDropWrapper};

mod cuda;
mod info;
mod kernel;
mod simulate;

mod almost_infinite;
mod in_memory;
mod non_spatial;

use crate::kernel::SimulationKernel;
use cuda::with_initialised_cuda;
use simulate::simulate;

pub struct CudaSimulation;

impl CudaSimulation {
    /// Simulates the coalescence algorithm on a CUDA-capable GPU on
    /// `habitat` with `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: HabitatToU64Injection + RustToCuda,
        D: DispersalSampler<H, CudaRng<Rng>> + RustToCuda,
        R: LineageReference<H> + rustacuda_core::DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        P: ReporterContext,
    >(
        habitat: H,
        dispersal_sampler: D,
        lineage_store: S,
        speciation_probability_per_generation: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64)> {
        const REPORT_SPECIATION: bool = true;
        const REPORT_DISPERSAL: bool = false;

        const SIMULATION_STEP_SLICE: usize = 100_usize;

        reporter_context.with_reporter(|reporter| {
            let rng = CudaRng::<Rng>::seed_from_u64(seed);
            let coalescence_sampler = CoalescenceSampler::default();
            let event_sampler = EventSampler::default();
            let active_lineage_sampler =
                ActiveLineageSampler::empty(PoissonEventTimeSampler::new(1.0_f64));

            let simulation = Simulation::builder()
                .speciation_probability_per_generation(speciation_probability_per_generation)
                .habitat(habitat)
                .rng(rng)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(std::marker::PhantomData::<R>)
                .lineage_store(lineage_store)
                .coalescence_sampler(coalescence_sampler)
                .event_sampler(event_sampler)
                .active_lineage_sampler(active_lineage_sampler)
                .build();

            // TODO: Need a way to tune these based on the available CUDA device or cmd args
            let block_size = BlockSize::xy(16, 16);
            let grid_size = GridSize::xy(16, 16);

            #[allow(clippy::cast_possible_truncation)]
            let grid_amount = {
                #[allow(clippy::cast_possible_truncation)]
                let total_individuals = simulation.lineage_store().get_number_total_lineages();

                let block_size = (block_size.x * block_size.y * block_size.z) as usize;
                let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;

                let task_size = block_size * grid_size;

                (total_individuals / task_size) + ((total_individuals % task_size > 0) as usize)
            } as u32;

            let (time, steps) = with_initialised_cuda(|| {
                let stream = CudaDropWrapper::from(Stream::new(StreamFlags::NON_BLOCKING, None)?);

                SimulationKernel::with_kernel(|kernel| {
                    let task_list = TaskListHost::new(&block_size, &grid_size)?;
                    let min_spec_sample_buffer = ValueBufferHost::new(&block_size, &grid_size)?;

                    #[allow(clippy::type_complexity)]
                    let event_buffer: EventBufferHost<
                        H,
                        R,
                        P::Reporter<H, R>,
                        { REPORT_SPECIATION },
                        { REPORT_DISPERSAL },
                    > = EventBufferHost::new(
                        reporter,
                        &block_size,
                        &grid_size,
                        SIMULATION_STEP_SLICE,
                    )?;

                    simulate(
                        &stream,
                        &kernel,
                        (grid_amount, grid_size, block_size),
                        simulation,
                        task_list,
                        event_buffer,
                        min_spec_sample_buffer,
                        SIMULATION_STEP_SLICE as u64,
                    )
                })
            })?;

            Ok((time, steps))
        })
    }
}