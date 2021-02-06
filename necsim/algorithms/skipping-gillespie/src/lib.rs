#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use std::marker::PhantomData;

use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageReference, RngCore, SeparableDispersalSampler},
    simulation::{partial::event_sampler::PartialSimulation, Simulation},
};

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        speciation_probability::uniform::UniformSpeciationProbability,
    },
    partitioning::LocalPartition,
};
use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::std::StdRng,
};

use necsim_impls_no_std::reporter::ReporterContext;

mod almost_infinite;
mod in_memory;
mod non_spatial;

pub struct SkippingGillespieSimulation;

impl SkippingGillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on the `habitat` with `dispersal` and lineages from
    /// `lineage_store`.
    fn simulate<
        H: Habitat,
        D: SeparableDispersalSampler<H, StdRng>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        P: ReporterContext,
        L: LocalPartition<P>,
    >(
        habitat_in: H,
        dispersal_sampler_in: D,
        lineage_store_in: S,
        speciation_probability_per_generation: f64,
        seed: u64,
        local_partition: &mut L,
    ) -> (f64, u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = ConditionalCoalescenceSampler::default();
        let event_sampler = ConditionalGillespieEventSampler::default();

        // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
        let partial_simulation = PartialSimulation {
            habitat: habitat_in,
            speciation_probability,
            dispersal_sampler: dispersal_sampler_in,
            lineage_reference: PhantomData::<R>,
            lineage_store: lineage_store_in,
            emigration_exit,
            coalescence_sampler,
            rng: PhantomData::<StdRng>,
        };

        let active_lineage_sampler =
            GillespieActiveLineageSampler::new(&partial_simulation, &event_sampler, &mut rng);

        // Unpack the PartialSimulation to create the full Simulation
        let PartialSimulation {
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_reference,
            lineage_store,
            emigration_exit,
            coalescence_sampler,
            rng: _,
        } = partial_simulation;

        let immigration_entry = NeverImmigrationEntry::default();

        let simulation = Simulation::builder()
            .habitat(habitat)
            .rng(rng)
            .speciation_probability(speciation_probability)
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(lineage_reference)
            .lineage_store(lineage_store)
            .emigration_exit(emigration_exit)
            .coalescence_sampler(coalescence_sampler)
            .event_sampler(event_sampler)
            .immigration_entry(immigration_entry)
            .active_lineage_sampler(active_lineage_sampler)
            .build();

        simulation.simulate(local_partition.get_reporter())
    }
}
