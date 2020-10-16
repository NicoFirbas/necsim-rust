#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Core {
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64)]
    fn sample_uniform(&mut self) -> f64;
}

pub trait Rng: Core {
    fn sample_index(&mut self, length: usize) -> usize {
        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = (self.sample_uniform() * (length as f64)).floor() as usize;
        index
    }

    fn sample_exponential(&mut self, lambda: f64) -> f64 {
        -self.sample_uniform().ln() / lambda
    }

    #[debug_requires(probability >= 0.0_f64 && probability <= 1.0_f64)]
    fn sample_event(&mut self, probability: f64) -> bool {
        self.sample_uniform() < probability
    }
}

impl<T: Core> Rng for T {}
