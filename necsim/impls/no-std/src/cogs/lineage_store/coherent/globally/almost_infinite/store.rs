use necsim_core::{
    cogs::{
        GloballyCoherentLineageStore, LineageStore, LocallyCoherentLineageStore, OriginSampler,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

use super::AlmostInfiniteLineageStore;

#[contract_trait]
impl LineageStore<AlmostInfiniteHabitat, InMemoryLineageReference> for AlmostInfiniteLineageStore {
    #[allow(clippy::type_complexity)]
    type LineageReferenceIterator<'a> = core::iter::Map<
        slab::Iter<'a, Lineage>,
        fn((usize, &'a Lineage)) -> InMemoryLineageReference,
    >;

    fn from_origin_sampler<'h, O: OriginSampler<'h, Habitat = AlmostInfiniteHabitat>>(
        origin_sampler: O,
    ) -> Self {
        Self::new(origin_sampler)
    }

    #[must_use]
    fn get_number_total_lineages(&self) -> usize {
        self.lineages_store.len()
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_> {
        self.lineages_store.iter().map(
            (|(reference, _)| InMemoryLineageReference::from(reference))
                as fn((usize, &'_ Lineage)) -> InMemoryLineageReference,
        )
    }

    #[must_use]
    fn get(&self, reference: InMemoryLineageReference) -> Option<&Lineage> {
        self.lineages_store.get(usize::from(reference))
    }
}

#[contract_trait]
impl LocallyCoherentLineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>
    for AlmostInfiniteLineageStore
{
    #[must_use]
    #[debug_requires(indexed_location.index() == 0, "only one lineage per location")]
    fn get_active_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &AlmostInfiniteHabitat,
    ) -> Option<&GlobalLineageReference> {
        self.location_to_lineage_references
            .get(indexed_location.location())
            .map(|local_reference| self[*local_reference].global_reference())
    }

    #[debug_requires(indexed_location.index() == 0, "only one lineage per location")]
    fn insert_lineage_to_indexed_location_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        indexed_location: IndexedLocation,
        _habitat: &AlmostInfiniteHabitat,
    ) {
        self.location_to_lineage_references
            .insert(indexed_location.location().clone(), reference);

        unsafe {
            self.lineages_store[usize::from(reference)].move_to_indexed_location(indexed_location);
        };
    }

    #[must_use]
    #[debug_requires(
        self[reference].indexed_location().unwrap().index() == 0,
        "only one lineage per location"
    )]
    fn extract_lineage_from_its_location_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        event_time: PositiveF64,
        _habitat: &AlmostInfiniteHabitat,
    ) -> (IndexedLocation, NonNegativeF64) {
        let lineage: &Lineage = &self.lineages_store[usize::from(reference)];

        let lineage_indexed_location = lineage.indexed_location().unwrap();

        let lineage_location = lineage_indexed_location.location();

        let lineage_reference_at_location = self
            .location_to_lineage_references
            .remove(lineage_location)
            .unwrap();

        unsafe {
            self.lineages_store[usize::from(lineage_reference_at_location)]
                .remove_from_location(event_time)
        }
    }

    fn emigrate(
        &mut self,
        local_lineage_reference: InMemoryLineageReference,
    ) -> GlobalLineageReference {
        self.lineages_store
            .remove(local_lineage_reference.into())
            .emigrate()
    }

    fn immigrate_locally_coherent(
        &mut self,
        _habitat: &AlmostInfiniteHabitat,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time_of_emigration: PositiveF64,
    ) -> InMemoryLineageReference {
        let location = indexed_location.location().clone();

        let lineage = Lineage::immigrate(global_reference, indexed_location, time_of_emigration);

        let local_lineage_reference =
            InMemoryLineageReference::from(self.lineages_store.insert(lineage));

        self.location_to_lineage_references
            .insert(location, local_lineage_reference);

        local_lineage_reference
    }
}

#[contract_trait]
impl GloballyCoherentLineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>
    for AlmostInfiniteLineageStore
{
    #[allow(clippy::type_complexity)]
    type LocationIterator<'a> = core::iter::Cloned<
        core::iter::FilterMap<
            slab::Iter<'a, Lineage>,
            fn((usize, &'a Lineage)) -> Option<&'a necsim_core::landscape::Location>,
        >,
    >;

    #[must_use]
    fn iter_active_locations(
        &self,
        _habitat: &AlmostInfiniteHabitat,
    ) -> Self::LocationIterator<'_> {
        self.lineages_store
            .iter()
            .filter_map(
                (|(_, lineage)| lineage.indexed_location().map(IndexedLocation::location))
                    as fn((usize, &'_ Lineage)) -> Option<&'_ Location>,
            )
            .cloned()
    }

    #[must_use]
    fn get_active_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        _habitat: &AlmostInfiniteHabitat,
    ) -> &[InMemoryLineageReference] {
        match self.location_to_lineage_references.get(location) {
            Some(local_reference) => core::slice::from_ref(local_reference),
            None => &[],
        }
    }
}
