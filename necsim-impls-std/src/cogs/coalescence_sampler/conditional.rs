use necsim_core::cogs::{CoalescenceSampler, Habitat, LineageReference, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
pub struct ConditionalCoalescenceSampler<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>(
    std::marker::PhantomData<(H, R, S)>,
);

impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>> Default
    for ConditionalCoalescenceSampler<H, R, S>
{
    fn default() -> Self {
        Self(std::marker::PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>> CoalescenceSampler<H, R, S>
    for ConditionalCoalescenceSampler<H, R, S>
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> Option<R> {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            rng,
        )
    }
}

impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>
    ConditionalCoalescenceSampler<H, R, S>
{
    #[must_use]
    pub fn sample_coalescence_at_location(
        location: &Location,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> R {
        let lineages_at_location = lineage_store.get_active_lineages_at_location(location);
        let population = lineages_at_location.len();

        let chosen_coalescence = rng.sample_index(population);

        lineages_at_location[chosen_coalescence].clone()
    }

    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(location) > 0, "location is habitable")]
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64, "returns probability")]
    pub fn get_coalescence_probability_at_location(
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        lineage_store_includes_self: bool,
    ) -> f64 {
        // If the lineage store includes self, the population must be decremented
        // to avoid coalescence with the self lineage

        #[allow(clippy::cast_precision_loss)]
        let population = (lineage_store
            .get_active_lineages_at_location(location)
            .len()
            - usize::from(lineage_store_includes_self)) as f64;
        let habitat = f64::from(habitat.get_habitat_at_location(location));

        population / habitat
    }
}
