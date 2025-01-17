#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]
#![feature(never_type)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

use necsim_core::cogs::{
    DispersalSampler, LineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    UniformlySampleableHabitat,
};
use necsim_core_bond::OpenClosedUnitF64 as PositiveUnitF64;
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::in_memory::InMemoryDispersalSampler,
        origin_sampler::{pre_sampler::OriginPreSampler, TrustedOriginSampler},
    },
    decomposition::Decomposition,
};

#[cfg(any(
    feature = "almost-infinite-normal-dispersal",
    feature = "almost-infinite-clark2dt-dispersal",
))]
pub mod almost_infinite;
#[cfg(feature = "non-spatial")]
pub mod non_spatial;
#[cfg(any(
    feature = "spatially-explicit-uniform-turnover",
    feature = "spatially-explicit-turnover-map"
))]
pub mod spatially_explicit;
#[cfg(feature = "spatially-implicit")]
pub mod spatially_implicit;
#[cfg(feature = "wrapping-noise")]
pub mod wrapping_noise;

pub trait ScenarioParameters {
    type Arguments;
    type Error;
}

pub trait Scenario<M: MathsCore, G: RngCore<M>>: Sized + ScenarioParameters {
    type Habitat: UniformlySampleableHabitat<M, G>;
    type OriginSampler<'h, I: Iterator<Item = u64>>: TrustedOriginSampler<
        'h,
        M,
        Habitat = Self::Habitat,
    >
    where
        M: 'h,
        G: 'h,
        Self: 'h;
    type OriginSamplerAuxiliary;
    type Decomposition: Decomposition<M, Self::Habitat>;
    type DecompositionAuxiliary;
    type LineageStore<L: LineageStore<M, Self::Habitat>>: LineageStore<M, Self::Habitat>;
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>>: DispersalSampler<
        M,
        Self::Habitat,
        G,
    >;
    type TurnoverRate: TurnoverRate<M, Self::Habitat>;
    type SpeciationProbability: SpeciationProbability<M, Self::Habitat>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the scenario failed
    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error>;

    /// Inside rustcoalescence, I know that only specialised
    /// `InMemoryDispersalSampler` implementations will be requested.
    #[allow(clippy::type_complexity)]
    fn build<D: InMemoryDispersalSampler<M, Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
        Self::OriginSamplerAuxiliary,
        Self::DecompositionAuxiliary,
    );

    fn sample_habitat<'h, I: Iterator<Item = u64>>(
        habitat: &'h Self::Habitat,
        pre_sampler: OriginPreSampler<M, I>,
        auxiliary: Self::OriginSamplerAuxiliary,
    ) -> Self::OriginSampler<'h, I>
    where
        G: 'h;

    fn decompose(
        habitat: &Self::Habitat,
        subdomain: Partition,
        auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition;
}
