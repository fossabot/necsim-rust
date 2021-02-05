use array2d::Array2D;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{
    partitioning::Partitioning, reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};

use super::ClassicalSimulation;

#[contract_trait]
impl InMemorySimulation for ClassicalSimulation {
    type AuxiliaryArguments = ();
    type Error = anyhow::Error;

    /// Simulates the classical coalescence algorithm on an in memory
    /// `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<P: Partitioning, R: ReporterContext>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        _partitioning: &mut P,
        reporter_context: R,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;

        let lineage_store = CoherentInMemoryLineageStore::new(InMemoryOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
        ));

        Ok(ClassicalSimulation::simulate(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_store,
            seed,
            reporter_context,
        ))
    }
}
