use core::{marker::PhantomData, num::NonZeroUsize};

use necsim_core::{
    cogs::{Backup, DispersalSampler, Habitat, RngCore, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

use crate::cogs::habitat::non_spatial::NonSpatialHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCuda))]
pub struct NonSpatialDispersalSampler<G: RngCore> {
    marker: PhantomData<G>,
}

impl<G: RngCore> Default for NonSpatialDispersalSampler<G> {
    #[must_use]
    fn default() -> Self {
        Self {
            marker: PhantomData::<G>,
        }
    }
}

#[contract_trait]
impl<G: RngCore> Backup for NonSpatialDispersalSampler<G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<G>,
        }
    }
}

#[contract_trait]
impl<G: RngCore> DispersalSampler<NonSpatialHabitat, G> for NonSpatialDispersalSampler<G> {
    #[must_use]
    #[inline]
    fn sample_dispersal_from_location(
        &self,
        _location: &Location,
        habitat: &NonSpatialHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let habitat_index_max =
            (habitat.get_extent().width() as usize) * (habitat.get_extent().height() as usize);

        // Safety: `habitat_index_max` is zero iff the habitat is 0x0, in which
        //         case the location cannot be in the landscape
        let dispersal_target_index =
            rng.sample_index(unsafe { NonZeroUsize::new_unchecked(habitat_index_max) });

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }
}

#[contract_trait]
impl<G: RngCore> SeparableDispersalSampler<NonSpatialHabitat, G> for NonSpatialDispersalSampler<G> {
    #[must_use]
    #[debug_requires((
        u64::from(habitat.get_extent().width()) * u64::from(habitat.get_extent().height())
    ) > 1_u64, "a different, non-self dispersal, target location exists")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &NonSpatialHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let habitat_index_max =
            (habitat.get_extent().width() as usize) * (habitat.get_extent().height() as usize);
        let current_location_index = (location.y() as usize)
            * (habitat.get_extent().width() as usize)
            + (location.x() as usize);

        // Safety: by PRE, `habitat_index_max` > 1
        let dispersal_target_index = {
            let dispersal_target_index =
                rng.sample_index(unsafe { NonZeroUsize::new_unchecked(habitat_index_max - 1) });

            if dispersal_target_index >= current_location_index {
                dispersal_target_index + 1
            } else {
                dispersal_target_index
            }
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }

    #[must_use]
    fn get_self_dispersal_probability_at_location(
        &self,
        _location: &Location,
        habitat: &NonSpatialHabitat,
    ) -> ClosedUnitF64 {
        let self_dispersal = 1.0_f64
            / (f64::from(habitat.get_extent().width()) * f64::from(habitat.get_extent().height()));

        // Safety: Since the method is only called for a valid location,
        //          width >= 1 and height >= 1
        //         => 1.0/(width*height) in [0.0; 1.0]
        unsafe { ClosedUnitF64::new_unchecked(self_dispersal) }
    }
}
