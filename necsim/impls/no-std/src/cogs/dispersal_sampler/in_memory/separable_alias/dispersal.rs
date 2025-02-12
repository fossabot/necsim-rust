use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

use super::InMemorySeparableAliasDispersalSampler;

#[contract_trait]
impl<H: Habitat, G: RngCore> DispersalSampler<H, G>
    for InMemorySeparableAliasDispersalSampler<H, G>
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let self_dispersal_at_location =
            self.get_self_dispersal_probability_at_location(location, habitat);

        if self_dispersal_at_location >= 1.0_f64 {
            return location.clone();
        }

        if self_dispersal_at_location > 0.0_f64 && rng.sample_event(self_dispersal_at_location) {
            return location.clone();
        }

        self.sample_non_self_dispersal_from_location(location, habitat, rng)
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore> SeparableDispersalSampler<H, G>
    for InMemorySeparableAliasDispersalSampler<H, G>
{
    #[must_use]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        let alias_dispersal_at_location = self.alias_dispersal[(
            (location.y() - habitat.get_extent().y()) as usize,
            (location.x() - habitat.get_extent().x()) as usize,
        )]
            .as_ref()
            .expect("habitat dispersal origin must disperse somewhere");

        let dispersal_target_index = alias_dispersal_at_location.sample_event(rng);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }

    #[must_use]
    #[debug_requires(habitat.get_extent().contains(location), "location is inside habitat extent")]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64 {
        self.self_dispersal[(
            (location.y() - habitat.get_extent().y()) as usize,
            (location.x() - habitat.get_extent().x()) as usize,
        )]
    }
}
