use anyhow::{Context, Result};

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

#[cfg(feature = "necsim-cuda")]
use necsim_cuda::CudaSimulation;

#[cfg(feature = "necsim-gillespie")]
use necsim_gillespie::GillespieSimulation;

#[cfg(feature = "necsim-skipping-gillespie")]
use necsim_skipping_gillespie::SkippingGillespieSimulation;

#[cfg(feature = "necsim-independent")]
use necsim_independent::IndependentSimulation;

use necsim_impls_no_std::reporter::ReporterContext;
#[allow(unused_imports)]
use necsim_impls_no_std::simulation::in_memory::InMemorySimulation;

use necsim_impls_no_std::partitioning::LocalPartition;

#[allow(unused_imports)]
use crate::args::{Algorithm, CommonArgs, InMemoryArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<R: ReporterContext, P: LocalPartition<R>>(
    common_args: CommonArgs,
    in_memory_args: InMemoryArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    info!(
        "Setting up the in-memory {} coalescence algorithm ...",
        common_args.algorithm
    );

    #[allow(clippy::match_single_binding)]
    let result: Result<(f64, u64)> = match common_args.algorithm {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            &in_memory_args.habitat_map,
            &in_memory_args.dispersal_map,
            common_args.speciation_probability_per_generation.get(),
            common_args.sample_percentage.get(),
            common_args.seed,
            local_partition,
            (),
        ),
        #[cfg(feature = "necsim-gillespie")]
        Algorithm::Gillespie => GillespieSimulation::simulate(
            &in_memory_args.habitat_map,
            &in_memory_args.dispersal_map,
            common_args.speciation_probability_per_generation.get(),
            common_args.sample_percentage.get(),
            common_args.seed,
            local_partition,
            (),
        ),
        #[cfg(feature = "necsim-skipping-gillespie")]
        Algorithm::SkippingGillespie(auxiliary) => SkippingGillespieSimulation::simulate(
            &in_memory_args.habitat_map,
            &in_memory_args.dispersal_map,
            common_args.speciation_probability_per_generation.get(),
            common_args.sample_percentage.get(),
            common_args.seed,
            local_partition,
            auxiliary,
        ),
        #[cfg(feature = "necsim-cuda")]
        Algorithm::Cuda(auxiliary) => CudaSimulation::simulate(
            &in_memory_args.habitat_map,
            &in_memory_args.dispersal_map,
            common_args.speciation_probability_per_generation.get(),
            common_args.sample_percentage.get(),
            common_args.seed,
            local_partition,
            auxiliary,
        ),
        #[cfg(feature = "necsim-independent")]
        Algorithm::Independent(auxiliary) => IndependentSimulation::simulate(
            &in_memory_args.habitat_map,
            &in_memory_args.dispersal_map,
            common_args.speciation_probability_per_generation.get(),
            common_args.sample_percentage.get(),
            common_args.seed,
            local_partition,
            auxiliary,
        ),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| "Failed to run the in-memory simulation.")
}
