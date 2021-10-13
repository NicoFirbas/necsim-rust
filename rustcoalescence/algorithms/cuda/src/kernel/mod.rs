use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, PrimeableRng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::MinSpeciationTrackingEventSampler,
};

use rust_cuda::{
    common::RustToCuda,
    host::{CudaDropWrapper, LaunchConfig, Launcher, TypedKernel},
    rustacuda::{
        error::CudaResult,
        function::{BlockSize, Function, GridSize},
        stream::Stream,
    },
};

use rustcoalescence_algorithms_cuda_kernel::Kernel;

mod link;

#[allow(clippy::type_complexity, clippy::module_name_repetitions)]
pub struct SimulationKernel<
    F: F64Core,
    H: Habitat<F> + RustToCuda,
    G: PrimeableRng<F> + RustToCuda,
    R: LineageReference<F, H>,
    S: LineageStore<F, H, R> + RustToCuda,
    X: EmigrationExit<F, H, G, R, S> + RustToCuda,
    D: DispersalSampler<F, H, G> + RustToCuda,
    C: CoalescenceSampler<F, H, R, S> + RustToCuda,
    T: TurnoverRate<F, H> + RustToCuda,
    N: SpeciationProbability<F, H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<F, H, G, R, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry<F> + RustToCuda,
    A: SingularActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
> {
    kernel: TypedKernel<
        dyn Kernel<F, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    >,
    stream: CudaDropWrapper<Stream>,
    grid: GridSize,
    block: BlockSize,
}

impl<
        F: F64Core,
        H: Habitat<F> + RustToCuda,
        G: PrimeableRng<F> + RustToCuda,
        R: LineageReference<F, H>,
        S: LineageStore<F, H, R> + RustToCuda,
        X: EmigrationExit<F, H, G, R, S> + RustToCuda,
        D: DispersalSampler<F, H, G> + RustToCuda,
        C: CoalescenceSampler<F, H, R, S> + RustToCuda,
        T: TurnoverRate<F, H> + RustToCuda,
        N: SpeciationProbability<F, H> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<F, H, G, R, S, X, D, C, T, N> + RustToCuda,
        I: ImmigrationEntry<F> + RustToCuda,
        A: SingularActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > SimulationKernel<F, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    pub fn try_new(stream: Stream, grid: GridSize, block: BlockSize) -> CudaResult<Self>
    where
        Self: Kernel<F, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    {
        let stream = CudaDropWrapper::from(stream);
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            stream,
            grid,
            block,
        })
    }
}

impl<
        F: F64Core,
        H: Habitat<F> + RustToCuda,
        G: PrimeableRng<F> + RustToCuda,
        R: LineageReference<F, H>,
        S: LineageStore<F, H, R> + RustToCuda,
        X: EmigrationExit<F, H, G, R, S> + RustToCuda,
        D: DispersalSampler<F, H, G> + RustToCuda,
        C: CoalescenceSampler<F, H, R, S> + RustToCuda,
        T: TurnoverRate<F, H> + RustToCuda,
        N: SpeciationProbability<F, H> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<F, H, G, R, S, X, D, C, T, N> + RustToCuda,
        I: ImmigrationEntry<F> + RustToCuda,
        A: SingularActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > Launcher
    for SimulationKernel<F, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    type KernelTraitObject =
        dyn Kernel<F, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>;

    fn get_config(&self) -> LaunchConfig {
        LaunchConfig {
            grid: self.grid.clone(),
            block: self.block.clone(),
            shared_memory_size: 0_u32,
        }
    }

    fn get_stream(&self) -> &Stream {
        &self.stream
    }

    fn get_kernel_mut(&mut self) -> &mut TypedKernel<Self::KernelTraitObject> {
        &mut self.kernel
    }

    fn on_compile(&mut self, kernel: &Function) -> CudaResult<()> {
        crate::info::print_kernel_function_attributes(kernel);

        Ok(())
    }
}
