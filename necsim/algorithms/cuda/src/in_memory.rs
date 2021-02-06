use array2d::Array2D;

use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
    origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};

use super::{CudaArguments, CudaSimulation};

#[contract_trait]
impl InMemorySimulation for CudaSimulation {
    type AuxiliaryArguments = CudaArguments;
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an in
    /// memory `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryPackedAliasDispersalSampler::new(dispersal, &habitat)?;

        let lineages = InMemoryOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
        )
        .map(|indexed_location| Lineage::new(indexed_location, &habitat))
        .collect();

        CudaSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineages,
            speciation_probability_per_generation,
            seed,
            local_partition,
            &auxiliary,
        )
    }
}
