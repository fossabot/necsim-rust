use alloc::{boxed::Box, vec::Vec};
use core::{marker::PhantomData, ops::Range};
use necsim_core_bond::NonNegativeF64;

use array2d::Array2D;

use necsim_core::{
    cogs::{Backup, Habitat, RngCore},
    landscape::Location,
};

mod dispersal;

use crate::alias::packed::AliasMethodSamplerAtom;

use super::InMemoryDispersalSampler;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
#[doc(hidden)]
pub struct AliasSamplerRange(Range<usize>);

impl From<Range<usize>> for AliasSamplerRange {
    fn from(range: Range<usize>) -> Self {
        Self(range)
    }
}

impl From<AliasSamplerRange> for Range<usize> {
    fn from(range: AliasSamplerRange) -> Self {
        range.0
    }
}

#[cfg(feature = "cuda")]
unsafe impl rust_cuda::rustacuda_core::DeviceCopy for AliasSamplerRange {}

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
pub struct InMemoryPackedAliasDispersalSampler<H: Habitat, G: RngCore> {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    alias_dispersal_ranges: Array2D<AliasSamplerRange>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    alias_dispersal_buffer: Box<[AliasMethodSamplerAtom<usize>]>,
    marker: PhantomData<(H, G)>,
}

#[contract_trait]
impl<H: Habitat, G: RngCore> InMemoryDispersalSampler<H, G>
    for InMemoryPackedAliasDispersalSampler<H, G>
{
    /// Creates a new `InMemoryPackedAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    fn unchecked_new(dispersal: &Array2D<NonNegativeF64>, habitat: &H) -> Self {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, NonNegativeF64)> =
            Vec::with_capacity(dispersal.row_len());

        let mut alias_dispersal_buffer = Vec::new();

        let alias_dispersal_ranges = Array2D::from_iter_row_major(
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

                let range_from = alias_dispersal_buffer.len();

                if event_weights.is_empty() {
                    AliasSamplerRange::from(range_from..range_from)
                } else {
                    alias_dispersal_buffer
                        .append(&mut AliasMethodSamplerAtom::create(&event_weights));

                    AliasSamplerRange::from(range_from..alias_dispersal_buffer.len())
                }
            }),
            habitat_extent.height() as usize,
            habitat_extent.width() as usize,
        )
        .unwrap(); // infallible by PRE;

        Self {
            alias_dispersal_ranges,
            alias_dispersal_buffer: alias_dispersal_buffer.into_boxed_slice(),
            marker: PhantomData::<(H, G)>,
        }
    }
}

impl<H: Habitat, G: RngCore> core::fmt::Debug for InMemoryPackedAliasDispersalSampler<H, G> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InMemoryPackedAliasDispersalSampler")
            .field("alias_dispersal_ranges", &self.alias_dispersal_ranges)
            .field(
                "alias_dispersal_buffer",
                &format_args!(
                    "Box [ {:p}; {} ]",
                    &self.alias_dispersal_buffer,
                    self.alias_dispersal_buffer.len()
                ),
            )
            .finish()
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore> Backup for InMemoryPackedAliasDispersalSampler<H, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_dispersal_ranges: self.alias_dispersal_ranges.clone(),
            alias_dispersal_buffer: self.alias_dispersal_buffer.clone(),
            marker: PhantomData::<(H, G)>,
        }
    }
}
