use necsim_core_bond::{NonNegativeF64, PositiveUnitF64};
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_plugins_core::import::AnyReporterPluginVec;

use crate::{
    args::{Algorithm, Partitioning, Sample, Scenario},
    cli::simulate::SimulationResult,
};

use super::super::BufferingSimulateArgsBuilder;

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub(in super::super) fn dispatch(
    _partitioning: Partitioning,
    _event_log: Option<EventLogRecorder>,
    _reporters: AnyReporterPluginVec,

    _speciation_probability_per_generation: PositiveUnitF64,
    _sample: Sample,
    _scenario: Scenario,
    _algorithm: Algorithm,
    _pause_before: Option<NonNegativeF64>,

    _ron_args: &str,
    _normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationResult> {
    extern "C" {
        fn simulate_dispatch_without_algorithm() -> !;
    }

    unsafe { simulate_dispatch_without_algorithm() }
}