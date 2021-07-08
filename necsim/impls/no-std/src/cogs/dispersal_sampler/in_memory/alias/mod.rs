use core::marker::PhantomData;

use alloc::vec::Vec;

use array2d::Array2D;

use necsim_core::{
    cogs::{Backup, Habitat, RngCore},
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

use crate::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

mod dispersal;

use crate::alias::AliasMethodSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemoryAliasDispersalSampler<H: Habitat, G: RngCore> {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    marker: PhantomData<(H, G)>,
}

#[contract_trait]
impl<H: Habitat, G: RngCore> InMemoryDispersalSampler<H, G>
    for InMemoryAliasDispersalSampler<H, G>
{
    /// Creates a new `InMemoryAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    fn unchecked_new(dispersal: &Array2D<NonNegativeF64>, habitat: &H) -> Self {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, NonNegativeF64)> =
            Vec::with_capacity(dispersal.row_len());

        let alias_dispersal = Array2D::from_iter_row_major(
            dispersal.rows_iter().map(|row| {
                event_weights.clear();

                for (col_index, dispersal_probability) in row.enumerate() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    // Multiply all dispersal probabilities by the habitat of their target
                    let weight = *dispersal_probability
                        * NonNegativeF64::from(habitat.get_habitat_at_location(&location));

                    if weight > 0.0_f64 {
                        event_weights.push((col_index, weight));
                    }
                }

                if event_weights.is_empty() {
                    None
                } else {
                    Some(AliasMethodSampler::new(&event_weights))
                }
            }),
            habitat_extent.height() as usize,
            habitat_extent.width() as usize,
        )
        .unwrap(); // infallible by PRE

        Self {
            alias_dispersal,
            marker: PhantomData::<(H, G)>,
        }
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore> Backup for InMemoryAliasDispersalSampler<H, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_dispersal: self.alias_dispersal.clone(),
            marker: PhantomData::<(H, G)>,
        }
    }
}
