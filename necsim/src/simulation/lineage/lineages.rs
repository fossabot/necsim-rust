use std::ops::Index;

use array2d::Array2D;

use super::{Lineage, LineageReference};

use crate::{
    landscape::{Landscape, Location},
    rng,
};

pub struct SimulationLineages {
    lineages_store: Vec<Lineage>,
    active_lineage_references: Vec<LineageReference>,
    location_to_lineage_references: Array2D<Vec<LineageReference>>,
}

impl SimulationLineages {
    #[must_use]
    pub fn new(landscape: &impl Landscape) -> Self {
        let mut lineages_store = Vec::with_capacity(landscape.get_total_habitat() as usize);

        let landscape_extent = landscape.get_extent();

        let mut location_to_lineage_references = Array2D::filled_with(
            Vec::new(),
            landscape_extent.height() as usize,
            landscape_extent.width() as usize,
        );

        for y in landscape_extent.y()..(landscape_extent.y() + landscape_extent.height()) {
            for x in landscape_extent.x()..(landscape_extent.x() + landscape_extent.width()) {
                let location = Location::new(x, y);

                let lineages_at_location =
                    &mut location_to_lineage_references[(y as usize, x as usize)];

                for index_at_location in 0..landscape.get_habitat_at_location(&location) {
                    lineages_at_location.push(LineageReference::new(lineages_store.len()));
                    lineages_store.push(Lineage::new(location.clone(), index_at_location as usize));
                }
            }
        }

        Self {
            active_lineage_references: (0..lineages_store.len())
                .map(LineageReference::new)
                .collect(),
            lineages_store,
            location_to_lineage_references,
        }
    }

    fn add_lineage_to_location(&mut self, reference: LineageReference, location: Location) {
        let lineages_at_location = &mut self.location_to_lineage_references
            [(location.y() as usize, location.x() as usize)];

        // TODO: We should assert that we never surpass the available habitat

        lineages_at_location.push(reference);

        self.lineages_store[reference.0].move_to_location(location, lineages_at_location.len());
    }

    fn remove_lineage_from_its_location(&mut self, reference: LineageReference) {
        let lineage = &self.lineages_store[reference.0];

        let lineages_at_location = &mut self.location_to_lineage_references[(
            lineage.location().y() as usize,
            lineage.location().x() as usize,
        )];

        if let Some(last_lineage_at_location) = lineages_at_location.pop() {
            let lineage_index_at_location = lineage.index_at_location();

            if lineage_index_at_location < lineages_at_location.len() {
                lineages_at_location[lineage_index_at_location] = last_lineage_at_location;

                self.lineages_store[last_lineage_at_location.0]
                    .update_index_at_location(lineage_index_at_location);
            }
        }
    }

    #[must_use]
    pub fn pop_random_active_lineage_reference(
        &mut self,
        rng: &mut impl rng::Rng,
    ) -> Option<LineageReference> {
        let last_active_lineage_reference = match self.active_lineage_references.pop() {
            Some(reference) => reference,
            None => return None,
        };

        let chosen_active_lineage_index =
            rng.sample_index(self.active_lineage_references.len() + 1);

        let chosen_lineage_reference =
            if chosen_active_lineage_index == self.active_lineage_references.len() {
                last_active_lineage_reference
            } else {
                self.active_lineage_references[chosen_active_lineage_index] =
                    last_active_lineage_reference;

                self.active_lineage_references[chosen_active_lineage_index]
            };

        self.remove_lineage_from_its_location(chosen_lineage_reference);

        Some(chosen_lineage_reference)
    }

    pub fn push_active_lineage_reference_at_location(
        &mut self,
        reference: LineageReference,
        location: Location,
    ) {
        self.add_lineage_to_location(reference, location);

        self.active_lineage_references.push(reference);
    }

    #[must_use]
    pub fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    #[must_use]
    pub fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl rng::Rng,
    ) -> Option<LineageReference> {
        let population = self.get_number_active_lineages_at_location(location);

        let chosen_coalescence = rng.sample_index(habitat as usize);

        if chosen_coalescence >= population {
            return None;
        }

        Some(
            self.location_to_lineage_references[(location.y() as usize, location.x() as usize)]
                [chosen_coalescence],
        )
    }

    #[must_use]
    pub fn get_number_active_lineages_at_location(&self, location: &Location) -> usize {
        self.location_to_lineage_references[(location.y() as usize, location.x() as usize)].len()
    }
}

impl Index<LineageReference> for SimulationLineages {
    type Output = Lineage;

    #[must_use]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference.0]
    }
}
