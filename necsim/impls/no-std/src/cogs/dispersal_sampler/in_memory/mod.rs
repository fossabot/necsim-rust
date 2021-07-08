use array2d::Array2D;

use necsim_core::cogs::{DispersalSampler, Habitat, RngCore};

pub mod contract;

pub mod alias;
pub mod cumulative;
pub mod packed_alias;
pub mod separable_alias;

use contract::explicit_in_memory_dispersal_check_contract;
use necsim_core_bond::NonNegativeF64;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait InMemoryDispersalSampler<H: Habitat, G: RngCore>: DispersalSampler<H, G> + Sized {
    #[debug_requires((
        dispersal.num_columns() == (
            (habitat.get_extent().width() * habitat.get_extent().height()) as usize
        ) && dispersal.num_rows() == (
            (habitat.get_extent().width() * habitat.get_extent().height()) as usize
        )
    ), "dispersal dimensions are consistent")]
    #[debug_requires(
        explicit_in_memory_dispersal_check_contract(dispersal, habitat),
        "dispersal probabilities are consistent"
    )]
    fn unchecked_new(dispersal: &Array2D<NonNegativeF64>, habitat: &H) -> Self;
}
