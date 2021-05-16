use std::{marker::PhantomData, num::NonZeroU32};

use array2d::Array2D;
use necsim_core::cogs::{DispersalSampler, Habitat, LineageStore, RngCore};

use necsim_core_bond::{NonNegativeF64, PositiveUnitF64};

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::in_memory::{
            contract::explicit_in_memory_dispersal_check_contract, InMemoryDispersalSampler,
        },
        habitat::in_memory::InMemoryHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::in_memory::{InMemoryTurnoverRate, InMemoryTurnoverRateError},
    },
    decomposition::equal::EqualDecomposition,
};

use necsim_impls_std::cogs::dispersal_sampler::in_memory::error::InMemoryDispersalSamplerError;

use crate::{Scenario, ScenarioParameters};

#[allow(clippy::module_name_repetitions)]
#[derive(thiserror::Error, displaydoc::Display, Debug)]
pub enum SpatiallyExplicitTurnoverScenarioError {
    /// invalid dispersal map: {0}
    DispersalMap(InMemoryDispersalSamplerError),
    /// invalid turnover map: {0}
    TurnoverMap(InMemoryTurnoverRateError),
}

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitTurnoverScenario<G: RngCore> {
    habitat: InMemoryHabitat,
    dispersal_map: Array2D<NonNegativeF64>,
    turnover_rate: InMemoryTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
    _marker: PhantomData<G>,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryTurnoverArguments {
    pub habitat_map: Array2D<u32>,
    pub dispersal_map: Array2D<NonNegativeF64>,
    pub turnover_map: Array2D<NonNegativeF64>,
}

impl<G: RngCore> ScenarioParameters for SpatiallyExplicitTurnoverScenario<G> {
    type Arguments = InMemoryTurnoverArguments;
    type Error = SpatiallyExplicitTurnoverScenarioError;
}

impl<G: RngCore> Scenario<G> for SpatiallyExplicitTurnoverScenario<G> {
    type Decomposition = EqualDecomposition<Self::Habitat>;
    type DispersalSampler<D: DispersalSampler<Self::Habitat, G>> = D;
    type Habitat = InMemoryHabitat;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<Self::Habitat, Self::LineageReference>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = InMemoryOriginSampler<'h, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = InMemoryTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = InMemoryHabitat::new(args.habitat_map);
        let turnover_rate = InMemoryTurnoverRate::new(args.turnover_map, &habitat)
            .map_err(SpatiallyExplicitTurnoverScenarioError::TurnoverMap)?;
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        let habitat_extent = habitat.get_extent();
        let habitat_area = (habitat_extent.width() as usize) * (habitat_extent.height() as usize);

        if args.dispersal_map.num_rows() != habitat_area
            || args.dispersal_map.num_columns() != habitat_area
        {
            return Err(SpatiallyExplicitTurnoverScenarioError::DispersalMap(
                InMemoryDispersalSamplerError::InconsistentDispersalMapSize,
            ));
        }

        if !explicit_in_memory_dispersal_check_contract(&args.dispersal_map, &habitat) {
            return Err(SpatiallyExplicitTurnoverScenarioError::DispersalMap(
                InMemoryDispersalSamplerError::InconsistentDispersalProbabilities,
            ));
        }

        Ok(Self {
            habitat,
            dispersal_map: args.dispersal_map,
            turnover_rate,
            speciation_probability,
            _marker: PhantomData::<G>,
        })
    }

    fn build<D: InMemoryDispersalSampler<Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    ) {
        let dispersal_sampler = D::unchecked_new(&self.dispersal_map, &self.habitat);

        (
            self.habitat,
            dispersal_sampler,
            self.turnover_rate,
            self.speciation_probability,
        )
    }

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<I>,
    ) -> Self::OriginSampler<'_, I> {
        InMemoryOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(
        habitat: &Self::Habitat,
        rank: u32,
        partitions: NonZeroU32,
    ) -> Self::Decomposition {
        match EqualDecomposition::weight(habitat, rank, partitions) {
            Ok(decomposition) => decomposition,
            Err(decomposition) => {
                warn!(
                    "Spatially explicit habitat of size {}x{} could not be partitioned into {} \
                     partition(s).",
                    habitat.get_extent().width(),
                    habitat.get_extent().height(),
                    partitions.get(),
                );

                decomposition
            },
        }
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}
