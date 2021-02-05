use core::{marker::PhantomData, ops::Index};

use alloc::vec::Vec;

use array2d::Array2D;
use hashbrown::hash_map::HashMap;

use necsim_core::{
    cogs::{Habitat, OriginSampler},
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CoherentInMemoryLineageStore<H: Habitat> {
    lineages_store: Vec<Lineage>,
    location_to_lineage_references: Array2D<Vec<InMemoryLineageReference>>,
    indexed_location_to_lineage_reference:
        HashMap<IndexedLocation, (GlobalLineageReference, usize)>,
    _marker: PhantomData<H>,
}

impl<H: Habitat> Index<InMemoryLineageReference> for CoherentInMemoryLineageStore<H> {
    type Output = Lineage;

    #[must_use]
    #[debug_requires(
        Into::<usize>::into(reference) < self.lineages_store.len(),
        "lineage reference is in range"
    )]
    fn index(&self, reference: InMemoryLineageReference) -> &Self::Output {
        &self.lineages_store[Into::<usize>::into(reference)]
    }
}

impl<'h, H: 'h + Habitat> CoherentInMemoryLineageStore<H> {
    #[must_use]
    pub fn new<O: OriginSampler<'h, Habitat = H>>(mut origin_sampler: O) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let lineages_amount_hint = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineages_store = Vec::with_capacity(lineages_amount_hint);

        let landscape_extent = origin_sampler.habitat().get_extent();

        let mut location_to_lineage_references = Array2D::filled_with(
            Vec::new(),
            landscape_extent.height() as usize,
            landscape_extent.width() as usize,
        );

        let mut indexed_location_to_lineage_reference =
            HashMap::with_capacity(lineages_amount_hint);

        let x_from = landscape_extent.x();
        let y_from = landscape_extent.y();

        while let Some(indexed_location) = origin_sampler.next() {
            let x_offset = indexed_location.location().x() - x_from;
            let y_offset = indexed_location.location().y() - y_from;

            let lineages_at_location =
                &mut location_to_lineage_references[(y_offset as usize, x_offset as usize)];

            let lineage = Lineage::new(indexed_location.clone(), origin_sampler.habitat());
            let local_reference = InMemoryLineageReference::from(lineages_store.len());

            indexed_location_to_lineage_reference.insert(
                indexed_location,
                (
                    lineage.global_reference().clone(),
                    lineages_at_location.len(),
                ),
            );
            lineages_at_location.push(local_reference);

            lineages_store.push(lineage);
        }

        lineages_store.shrink_to_fit();

        Self {
            lineages_store,
            location_to_lineage_references,
            indexed_location_to_lineage_reference,
            _marker: PhantomData::<H>,
        }
    }
}
