use rustcoalescence_algorithms_cuda_kernel::{Kernel, KernelArgs};

use super::SimulationKernel;

macro_rules! link_kernel {
    ($habitat:ty, $dispersal:ty, $turnover:ty, $speciation:ty) => {
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::False,
            necsim_core::reporter::boolean::False
        }
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::False,
            necsim_core::reporter::boolean::True
        }
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::True,
            necsim_core::reporter::boolean::False
        }
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::True,
            necsim_core::reporter::boolean::True
        }
    };
    (
        $habitat:ty, $dispersal:ty, $turnover:ty, $speciation:ty,
        $report_speciation:ty, $report_dispersal:ty
    ) => {
        rustcoalescence_algorithms_cuda_kernel::link_kernel!(
            $habitat,
            necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
            necsim_core::lineage::GlobalLineageReference,
            necsim_impls_no_std::cogs::lineage_store::independent::IndependentLineageStore<
                $habitat
            >,
            necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
            $dispersal,
            necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler<
                $habitat,
            >,
            $turnover,
            $speciation,
            necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler<
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                $dispersal,
                $turnover,
                $speciation,
            >,
            necsim_impls_no_std::cogs::immigration_entry::never::NeverImmigrationEntry,
            necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler<
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                $dispersal,
                $turnover,
                $speciation,
                necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::exp::ExpEventTimeSampler,
            >,
            $report_speciation,
            $report_dispersal,
        );
    };
}

link_kernel!(
    necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat,
    necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

link_kernel!(
    necsim_impls_no_std::cogs::habitat::spatially_implicit::SpatiallyImplicitHabitat,
    necsim_impls_no_std::cogs::dispersal_sampler::spatially_implicit::SpatiallyImplicitDispersalSampler<
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::spatially_implicit::SpatiallyImplicitSpeciationProbability
);

link_kernel!(
    necsim_impls_no_std::cogs::habitat::almost_infinite::AlmostInfiniteHabitat,
    necsim_impls_no_std::cogs::dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler<
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

link_kernel!(
    necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat,
    necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler<
        necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat,
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);