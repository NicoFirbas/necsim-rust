use necsim_impls_no_std::cogs::maths::intrinsics::IntrinsicsMathsCore;

use rustcoalescence_algorithms::{AlgorithmDefaults, AlgorithmParamters};

use crate::arguments::GillespieArguments;

mod classical;
mod turnover;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum GillespieAlgorithm {}

impl AlgorithmParamters for GillespieAlgorithm {
    type Arguments = GillespieArguments;
    type Error = !;
}

impl AlgorithmDefaults for GillespieAlgorithm {
    type MathsCore = IntrinsicsMathsCore;
}
