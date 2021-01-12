use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
        LineageReference, LineageStore, RngCore, SpeciationProbability,
    },
    event::{Event, EventType},
    landscape::IndexedLocation,
    simulation::partial::event_sampler::PartialSimulation,
};

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(Debug)]
pub struct UnconditionalEventSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, G, R, S>,
>(PhantomData<(H, G, N, D, R, S, X, C)>);

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, G, R, S>,
    > Default for UnconditionalEventSampler<H, G, N, D, R, S, X, C>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, N, D, R, S, X, C)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, G, R, S>,
    > EventSampler<H, G, N, D, R, S, X, C> for UnconditionalEventSampler<H, G, N, D, R, S, X, C>
{
    #[must_use]
    #[allow(clippy::shadow_unrelated)] // https://github.com/rust-lang/rust-clippy/issues/5455
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C>,
        rng: &mut G,
    ) -> Option<Event> {
        use necsim_core::cogs::RngSampler;

        let dispersal_origin = indexed_location;

        let (event_type, lineage_reference, dispersal_origin, event_time) = if rng.sample_event(
            simulation
                .speciation_probability
                .get_speciation_probability_at_location(dispersal_origin.location()),
        ) {
            (
                EventType::Speciation,
                lineage_reference,
                dispersal_origin,
                event_time,
            )
        } else {
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(dispersal_origin.location(), rng);

            // Check for emigration and return None iff lineage emigrated
            let (lineage_reference, dispersal_origin, dispersal_target, event_time) = simulation
                .with_mut_split_emigration_exit(|emigration_exit, simulation| {
                    emigration_exit.optionally_emigrate(
                        lineage_reference,
                        dispersal_origin,
                        dispersal_target,
                        event_time,
                        simulation,
                        rng,
                    )
                })?;

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    rng,
                );

            (
                EventType::Dispersal {
                    coalescence: optional_coalescence,
                    target: dispersal_target,
                },
                lineage_reference,
                dispersal_origin,
                event_time,
            )
        };

        Some(Event::new(
            dispersal_origin,
            event_time,
            simulation.lineage_store[lineage_reference]
                .global_reference()
                .clone(),
            event_type,
        ))
    }
}
