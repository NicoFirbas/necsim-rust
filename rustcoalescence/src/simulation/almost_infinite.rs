use anyhow::{Context, Result};

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

#[cfg(feature = "necsim-cuda")]
use necsim_cuda::CudaSimulation;

#[cfg(feature = "necsim-gillespie")]
use necsim_gillespie::GillespieSimulation;

#[cfg(feature = "necsim-skipping-gillespie")]
use necsim_skipping_gillespie::SkippingGillespieSimulation;

use necsim_impls_no_std::reporter::ReporterContext;
#[allow(unused_imports)]
use necsim_impls_no_std::simulation::almost_infinite::AlmostInfiniteSimulation;

#[allow(unused_imports)]
use crate::args::{Algorithm, AlmostInfiniteArgs, CommonArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<P: ReporterContext>(
    common_args: &CommonArgs,
    almost_infinite_args: &AlmostInfiniteArgs,
    reporter_context: P,
) -> Result<(f64, u64)> {
    println!(
        "Setting up the almost-infinite {:?} coalescence algorithm ...",
        common_args.algorithm()
    );

    #[allow(clippy::match_single_binding)]
    #[allow(clippy::map_err_ignore)]
    let result: Result<(f64, u64)> = match common_args.algorithm() {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        )
        .map_err(|_| unreachable!("Almost-Infinite ClassicalSimulation can never fail.")),
        #[cfg(feature = "necsim-gillespie")]
        Algorithm::Gillespie => GillespieSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        )
        .map_err(|_| unreachable!("Almost-Infinite GillespieSimulation can never fail.")),
        #[cfg(feature = "necsim-skipping-gillespie")]
        Algorithm::SkippingGillespie => SkippingGillespieSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        )
        .map_err(|_| unreachable!("Almost-Infinite SkppingGillespieSimulation can never fail.")),
        #[cfg(feature = "necsim-cuda")]
        Algorithm::CUDA => CudaSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| {
        format!(
            "Failed to run the almost-infinite simulation with radius {:?} and sigma {:?}.",
            almost_infinite_args.radius(),
            almost_infinite_args.sigma()
        )
    })
}