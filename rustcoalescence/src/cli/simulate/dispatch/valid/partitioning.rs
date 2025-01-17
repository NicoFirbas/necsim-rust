use necsim_core_bond::{NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::Partitioning as _;
use necsim_plugins_core::{import::AnyReporterPluginVec, match_any_reporter_plugin_vec};

use necsim_partitioning_monolithic::MonolithicLocalPartition;
#[cfg(feature = "necsim-partitioning-mpi")]
use necsim_partitioning_mpi::MpiLocalPartition;

use crate::{
    args::config::{
        algorithm::Algorithm, partitioning::Partitioning, sample::Sample, scenario::Scenario,
    },
    cli::simulate::SimulationOutcome,
    reporter::DynamicReporterContext,
};

use super::{super::super::BufferingSimulateArgsBuilder, algorithm_scenario};

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    partitioning: Partitioning,
    event_log: Option<EventLogRecorder>,
    reporters: AnyReporterPluginVec,

    speciation_probability_per_generation: PositiveUnitF64,
    sample: Sample,
    scenario: Scenario,
    algorithm: Algorithm,
    pause_before: Option<NonNegativeF64>,

    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome> {
    match_any_reporter_plugin_vec!(reporters => |reporter| {
        // Initialise the local partition and the simulation
        match partitioning {
            Partitioning::Monolithic(partitioning) => partitioning.with_local_partition(
                DynamicReporterContext::new(reporter), event_log, |partition| match partition {
                    MonolithicLocalPartition::Live(partition) => algorithm_scenario::dispatch(
                        *partition, speciation_probability_per_generation, sample, scenario,
                        algorithm, pause_before, ron_args, normalised_args,
                    ),
                    MonolithicLocalPartition::Recorded(partition) => algorithm_scenario::dispatch(
                        *partition, speciation_probability_per_generation, sample, scenario,
                        algorithm, pause_before, ron_args, normalised_args,
                    ),
                },
            ),
            #[cfg(feature = "necsim-partitioning-mpi")]
            Partitioning::Mpi(partitioning) => partitioning.with_local_partition(
                DynamicReporterContext::new(reporter), event_log, |partition| match partition {
                    MpiLocalPartition::Root(partition) => algorithm_scenario::dispatch(
                        *partition, speciation_probability_per_generation, sample, scenario,
                        algorithm, pause_before, ron_args, normalised_args,
                    ),
                    MpiLocalPartition::Parallel(partition) => algorithm_scenario::dispatch(
                        *partition, speciation_probability_per_generation, sample, scenario,
                        algorithm, pause_before, ron_args, normalised_args,
                    ),
                },
            ),
        }.flatten()
    })
}
