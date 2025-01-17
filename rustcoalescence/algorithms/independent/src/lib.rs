#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate serde_derive_state;

use necsim_core::{cogs::MathsCore, lineage::Lineage, reporter::Reporter};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::{
    lineage_store::independent::IndependentLineageStore, maths::intrinsics::IntrinsicsMathsCore,
    origin_sampler::pre_sampler::OriginPreSampler, rng::wyhash::WyHash,
};
use necsim_partitioning_core::{partition::Partition, LocalPartition};

use rustcoalescence_algorithms::{
    result::{ResumeError, SimulationOutcome},
    strategy::RestartFixUpStrategy,
    Algorithm, AlgorithmDefaults, AlgorithmParamters,
};
use rustcoalescence_scenarios::Scenario;

mod arguments;
mod initialiser;
mod launch;

use arguments::{IndependentArguments, IsolatedParallelismMode, ParallelismMode};
use initialiser::{
    fixup::FixUpInitialiser, genesis::GenesisInitialiser, resume::ResumeInitialiser,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum IndependentAlgorithm {}

impl AlgorithmParamters for IndependentAlgorithm {
    type Arguments = IndependentArguments;
    type Error = !;
}

impl AlgorithmDefaults for IndependentAlgorithm {
    type MathsCore = IntrinsicsMathsCore;
}

impl<'p, O: Scenario<M, WyHash<M>>, R: Reporter, P: LocalPartition<'p, R>, M: MathsCore>
    Algorithm<'p, M, O, R, P> for IndependentAlgorithm
{
    type LineageStore = IndependentLineageStore<M, O::Habitat>;
    type Rng = WyHash<M>;

    fn get_logical_partition(args: &Self::Arguments, local_partition: &P) -> Partition {
        match &args.parallelism_mode {
            ParallelismMode::Monolithic(_) => Partition::monolithic(),
            ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { partition, .. })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { partition, .. }) => {
                *partition
            },
            ParallelismMode::Individuals
            | ParallelismMode::Landscape
            | ParallelismMode::Probabilistic(_) => local_partition.get_partition(),
        }
    }

    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, Self::Error> {
        launch::initialise_and_simulate(
            &args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            GenesisInitialiser,
        )
    }

    /// # Errors
    ///
    /// Returns a `ContinueError::Sample` if initialising the resuming
    ///  simulation failed
    fn resume_and_simulate<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        lineages: L,
        resume_after: Option<NonNegativeF64>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, ResumeError<Self::Error>> {
        launch::initialise_and_simulate(
            &args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            ResumeInitialiser {
                lineages,
                resume_after,
            },
        )
    }

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if fixing up the restarting
    ///  simulation (incl. running the algorithm) failed
    #[allow(clippy::too_many_lines)]
    fn fixup_for_restart<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        lineages: L,
        restart_at: PositiveF64,
        fixup_strategy: RestartFixUpStrategy,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, ResumeError<Self::Error>> {
        launch::initialise_and_simulate(
            &args,
            rng,
            scenario,
            pre_sampler,
            Some(PositiveF64::max_after(restart_at.into(), restart_at.into()).into()),
            local_partition,
            FixUpInitialiser {
                lineages,
                restart_at,
                fixup_strategy,
            },
        )
    }
}
