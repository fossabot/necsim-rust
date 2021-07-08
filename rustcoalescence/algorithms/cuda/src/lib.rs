#![deny(clippy::pedantic)]
#![feature(option_result_unwrap_unchecked)]
#![feature(drain_filter)]

#[macro_use]
extern crate serde_derive_state;

use std::collections::VecDeque;

use necsim_core::{
    cogs::RngCore,
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::Simulation,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_cuda::cogs::rng::CudaRng;
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::{decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler},
    rng::wyhash::WyHash,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{Algorithm, AlgorithmParamters};
use rustcoalescence_scenarios::Scenario;

use rust_cuda::{
    common::RustToCuda,
    host::CudaDropWrapper,
    rustacuda::{
        function::{BlockSize, GridSize},
        prelude::{Stream, StreamFlags},
    },
};

mod arguments;
mod cuda;
mod info;
mod kernel;
mod parallelisation;

use arguments::{
    CudaArguments, IsolatedParallelismMode, MonolithicParallelismMode, ParallelismMode,
};

use crate::kernel::SimulationKernel;
use cuda::with_initialised_cuda;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum CudaAlgorithm {}

impl AlgorithmParamters for CudaAlgorithm {
    type Arguments = CudaArguments;
    type Error = anyhow::Error;
}

#[allow(clippy::type_complexity)]
impl<O: Scenario<CudaRng<WyHash>>> Algorithm<O> for CudaAlgorithm
where
    O::Habitat: RustToCuda,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<O::Habitat, CudaRng<WyHash>>>:
        RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
{
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<O::Habitat>;
    type Rng = CudaRng<WyHash>;

    fn initialise_and_simulate<I: Iterator<Item = u64>, R: Reporter, P: LocalPartition<R>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<I>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        let lineages: VecDeque<Lineage> = match args.parallelism_mode {
            // Apply no lineage origin partitioning in the `Monolithic` mode
            ParallelismMode::Monolithic(..) => scenario
                .sample_habitat(pre_sampler)
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect(),
            // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
            ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { partition, .. }) => {
                scenario
                    .sample_habitat(
                        pre_sampler.partition(partition.rank(), partition.partitions().get()),
                    )
                    .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                    .collect()
            },
            // Apply lineage origin partitioning in the `IsolatedLandscape` mode
            ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { partition, .. }) => {
                DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &O::decompose(scenario.habitat(), partition.rank(), partition.partitions()),
                )
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect()
            },
        };

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
            scenario.build::<InMemoryPackedAliasDispersalSampler<O::Habitat, CudaRng<WyHash>>>();
        let rng = CudaRng::from(WyHash::seed_from_u64(seed));
        let lineage_store = IndependentLineageStore::default();
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();

        let active_lineage_sampler =
            IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(args.delta_t));

        let simulation = Simulation::builder()
            .habitat(habitat)
            .rng(rng)
            .speciation_probability(speciation_probability)
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
            .lineage_store(lineage_store)
            .emigration_exit(emigration_exit)
            .coalescence_sampler(coalescence_sampler)
            .turnover_rate(turnover_rate)
            .event_sampler(event_sampler)
            .immigration_entry(immigration_entry)
            .active_lineage_sampler(active_lineage_sampler)
            .build();

        // Note: It seems to be more performant to spawn smaller blocks
        let block_size = BlockSize::x(args.block_size);
        let grid_size = GridSize::x(args.grid_size);

        let event_slice = match args.parallelism_mode {
            ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
            | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                event_slice, ..
            })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
                event_slice
            },
        };

        with_initialised_cuda(args.device, || {
            let stream = CudaDropWrapper::from(Stream::new(StreamFlags::NON_BLOCKING, None)?);

            SimulationKernel::with_kernel(args.ptx_jit, |kernel| {
                info::print_kernel_function_attributes(kernel.function());

                parallelisation::monolithic::simulate(
                    simulation,
                    kernel,
                    &stream,
                    (grid_size, block_size, args.dedup_cache, args.step_slice),
                    lineages,
                    event_slice,
                    local_partition,
                )
            })
        })
    }
}
