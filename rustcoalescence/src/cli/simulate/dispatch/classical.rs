use std::any::type_name;

use anyhow::Result;

use necsim_impls_std::cogs::rng::pcg::Pcg;
use rustcoalescence_algorithms::{Algorithm, AlgorithmParamters};

#[cfg(feature = "rustcoalescence-algorithms-monolithic")]
use rustcoalescence_algorithms_monolithic::classical::ClassicalAlgorithm;

use necsim_core::{
    cogs::{Habitat, LocallyCoherentLineageStore, RngCore, TurnoverRate},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;
use necsim_impls_no_std::cogs::{
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::locally::classical::ClassicalLineageStore,
    origin_sampler::pre_sampler::OriginPreSampler, turnover_rate::uniform::UniformTurnoverRate,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

#[allow(clippy::module_name_repetitions)]
pub trait ClassicalAlgorithmTurnoverDispatch<
    H: Habitat,
    T: TurnoverRate<H>,
    O: Scenario<Self::Rng, Habitat = H, TurnoverRate = T>,
>
{
    type Rng: RngCore;
    type Arguments;
    type Error;

    fn initialise_and_simulate_dispatch<
        I: Iterator<Item = u64>,
        R: Reporter,
        P: LocalPartition<R>,
    >(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<I>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error>;
}

impl<
        H: Habitat,
        T: TurnoverRate<H>,
        O: Scenario<Pcg, Habitat = H, TurnoverRate = T, LineageReference = InMemoryLineageReference>,
    > ClassicalAlgorithmTurnoverDispatch<H, T, O> for ClassicalAlgorithm
{
    type Arguments = <Self as AlgorithmParamters>::Arguments;
    type Error = anyhow::Error;
    type Rng = Pcg;

    default fn initialise_and_simulate_dispatch<
        I: Iterator<Item = u64>,
        R: Reporter,
        P: LocalPartition<R>,
    >(
        _args: Self::Arguments,
        _seed: u64,
        _scenario: O,
        _pre_sampler: OriginPreSampler<I>,
        _local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        anyhow::bail!(format!(
            "The Classical Algorithm only supports a UniformTurnoverRate, but not {}.",
            type_name::<T>()
        ))
    }
}

impl<
        H: Habitat,
        O: Scenario<
            Pcg,
            Habitat = H,
            TurnoverRate = UniformTurnoverRate,
            LineageReference = InMemoryLineageReference,
        >,
    > ClassicalAlgorithmTurnoverDispatch<H, UniformTurnoverRate, O> for ClassicalAlgorithm
where
    O::LineageStore<ClassicalLineageStore<O::Habitat>>:
        LocallyCoherentLineageStore<O::Habitat, InMemoryLineageReference>,
{
    fn initialise_and_simulate_dispatch<
        I: Iterator<Item = u64>,
        R: Reporter,
        P: LocalPartition<R>,
    >(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<I>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        Ok(ClassicalAlgorithm::initialise_and_simulate(
            args,
            seed,
            scenario,
            pre_sampler,
            local_partition,
        )
        .into_ok())
    }
}
