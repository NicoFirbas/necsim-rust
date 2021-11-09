use std::{convert::TryFrom, marker::PhantomData, path::PathBuf};

use serde::{Deserialize, Serialize, Serializer};

use necsim_core::cogs::{DispersalSampler, Habitat, LineageStore, MathsCore, RngCore};
use necsim_core_bond::{NonNegativeF64, Partition, PositiveF64, PositiveUnitF64};

use necsim_impls_no_std::{
    array2d::Array2D,
    cogs::{
        dispersal_sampler::in_memory::{
            contract::explicit_in_memory_dispersal_check_contract, InMemoryDispersalSampler,
        },
        habitat::in_memory::InMemoryHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::equal::EqualDecomposition,
};

use necsim_impls_std::cogs::dispersal_sampler::in_memory::error::InMemoryDispersalSamplerError;

use crate::{Scenario, ScenarioParameters};

use super::super::maps::{self, MapLoadingMode};

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitUniformTurnoverScenario<M: MathsCore, G: RngCore<M>> {
    habitat: InMemoryHabitat<M>,
    dispersal_map: Array2D<NonNegativeF64>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
    _marker: PhantomData<G>,
}

impl<M: MathsCore, G: RngCore<M>> ScenarioParameters
    for SpatiallyExplicitUniformTurnoverScenario<M, G>
{
    type Arguments = SpatiallyExplicitUniformTurnoverArguments;
    type Error = InMemoryDispersalSamplerError;
}

impl<M: MathsCore, G: RngCore<M>> Scenario<M, G>
    for SpatiallyExplicitUniformTurnoverScenario<M, G>
{
    type Decomposition = EqualDecomposition<M, Self::Habitat>;
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> = D;
    type Habitat = InMemoryHabitat<M>;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<M, Self::Habitat, Self::LineageReference>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = InMemoryOriginSampler<'h, M, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = InMemoryHabitat::new(args.habitat_map);
        let turnover_rate = UniformTurnoverRate::new(args.turnover_rate);
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        let habitat_extent = habitat.get_extent();
        let habitat_area = (habitat_extent.width() as usize) * (habitat_extent.height() as usize);

        if args.dispersal_map.num_rows() != habitat_area
            || args.dispersal_map.num_columns() != habitat_area
        {
            return Err(InMemoryDispersalSamplerError::InconsistentDispersalMapSize);
        }

        if !explicit_in_memory_dispersal_check_contract(&args.dispersal_map, &habitat) {
            return Err(InMemoryDispersalSamplerError::InconsistentDispersalProbabilities);
        }

        Ok(Self {
            habitat,
            dispersal_map: args.dispersal_map,
            turnover_rate,
            speciation_probability,
            _marker: PhantomData::<G>,
        })
    }

    fn build<D: InMemoryDispersalSampler<M, Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    ) {
        let dispersal_sampler = D::unchecked_new(&self.dispersal_map, &self.habitat);

        (
            self.habitat,
            dispersal_sampler,
            self.turnover_rate,
            self.speciation_probability,
        )
    }

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<M, I>,
    ) -> Self::OriginSampler<'_, I> {
        InMemoryOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(habitat: &Self::Habitat, subdomain: Partition) -> Self::Decomposition {
        match EqualDecomposition::weight(habitat, subdomain) {
            Ok(decomposition) => decomposition,
            Err(decomposition) => {
                warn!(
                    "Spatially explicit habitat of size {}x{} could not be partitioned into {} \
                     partition(s).",
                    habitat.get_extent().width(),
                    habitat.get_extent().height(),
                    subdomain.size().get(),
                );

                decomposition
            },
        }
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "SpatiallyExplicitUniformTurnoverArgumentsRaw")]
#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitUniformTurnoverArguments {
    pub habitat_path: PathBuf,
    pub habitat_map: Array2D<u32>,
    pub dispersal_path: PathBuf,
    pub dispersal_map: Array2D<NonNegativeF64>,
    pub turnover_rate: PositiveF64,
    pub loading_mode: MapLoadingMode,
}

impl Serialize for SpatiallyExplicitUniformTurnoverArguments {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SpatiallyExplicitUniformTurnoverArgumentsRaw {
            habitat_map: self.habitat_path.clone(),
            dispersal_map: self.dispersal_path.clone(),
            turnover_rate: self.turnover_rate,
            loading_mode: self.loading_mode,
        }
        .serialize(serializer)
    }
}

impl TryFrom<SpatiallyExplicitUniformTurnoverArgumentsRaw>
    for SpatiallyExplicitUniformTurnoverArguments
{
    type Error = String;

    fn try_from(raw: SpatiallyExplicitUniformTurnoverArgumentsRaw) -> Result<Self, Self::Error> {
        info!(
            "Starting to load the dispersal map {:?} ...",
            &raw.dispersal_map
        );

        let mut dispersal_map = maps::load_dispersal_map(&raw.dispersal_map, raw.loading_mode)
            .map_err(|err| format!("{:?}", err))?;

        info!(
            "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
            &raw.dispersal_map,
            dispersal_map.num_columns(),
            dispersal_map.num_rows()
        );

        info!(
            "Starting to load the habitat map {:?} ...",
            &raw.habitat_map
        );

        let habitat_map =
            maps::load_habitat_map(&raw.habitat_map, None, &mut dispersal_map, raw.loading_mode)
                .map_err(|err| format!("{:?}", err))?;

        info!(
            "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
            &raw.habitat_map,
            habitat_map.num_columns(),
            habitat_map.num_rows()
        );

        Ok(SpatiallyExplicitUniformTurnoverArguments {
            habitat_path: raw.habitat_map,
            habitat_map,
            dispersal_path: raw.dispersal_map,
            dispersal_map,
            turnover_rate: raw.turnover_rate,
            loading_mode: raw.loading_mode,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SpatiallyExplicitUniformTurnoverArguments")]
struct SpatiallyExplicitUniformTurnoverArgumentsRaw {
    #[serde(rename = "habitat", alias = "habitat_map")]
    habitat_map: PathBuf,

    #[serde(rename = "dispersal", alias = "dispersal_map")]
    dispersal_map: PathBuf,

    #[serde(rename = "turnover", alias = "turnover_rate")]
    #[serde(default = "default_turnover_rate")]
    turnover_rate: PositiveF64,

    #[serde(default)]
    #[serde(rename = "mode", alias = "loading_mode")]
    loading_mode: MapLoadingMode,
}

fn default_turnover_rate() -> PositiveF64 {
    PositiveF64::new(0.5_f64).unwrap()
}