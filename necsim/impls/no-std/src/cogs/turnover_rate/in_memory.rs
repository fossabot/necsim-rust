use alloc::boxed::Box;

use array2d::Array2D;

use necsim_core::{
    cogs::{Backup, Habitat, TurnoverRate},
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

use crate::cogs::habitat::in_memory::InMemoryHabitat;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct InMemoryTurnoverRate {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    turnover_rate: Box<[NonNegativeF64]>,
}

#[contract_trait]
impl Backup for InMemoryTurnoverRate {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            turnover_rate: self.turnover_rate.clone(),
        }
    }
}

#[contract_trait]
impl TurnoverRate<InMemoryHabitat> for InMemoryTurnoverRate {
    #[must_use]
    #[inline]
    fn get_turnover_rate_at_location(
        &self,
        location: &Location,
        habitat: &InMemoryHabitat,
    ) -> NonNegativeF64 {
        let extent = habitat.get_extent();

        self.turnover_rate
            .get(
                ((location.y() - extent.y()) as usize) * (extent.width() as usize)
                    + ((location.x() - extent.x()) as usize),
            )
            .copied()
            .unwrap_or_else(NonNegativeF64::zero)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(displaydoc::Display, Debug)]
pub enum InMemoryTurnoverRateError {
    /// There is some location with zero turnover and non-zero habitat.
    ZeroTurnoverHabitat,
}

impl InMemoryTurnoverRate {
    /// # Errors
    ///
    /// Returns `InMemoryTurnoverRateError::ZeroTurnoverHabitat` iff there is
    ///  zero turnover at any location with non-zero habitat.
    pub fn new(
        turnover_rate: Array2D<NonNegativeF64>,
        habitat: &InMemoryHabitat,
    ) -> Result<Self, InMemoryTurnoverRateError> {
        if habitat
            .get_extent()
            .iter()
            .zip(turnover_rate.elements_row_major_iter())
            .all(|(location, turnover)| {
                (*turnover != 0.0_f64) || (habitat.get_habitat_at_location(&location) == 0)
            })
        {
            Ok(Self {
                turnover_rate: turnover_rate.into_row_major().into_boxed_slice(),
            })
        } else {
            Err(InMemoryTurnoverRateError::ZeroTurnoverHabitat)
        }
    }
}
