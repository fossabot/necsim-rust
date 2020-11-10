use core::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
    RngCore,
};
use necsim_core::event::{Event, EventType};
use necsim_core::landscape::IndexedLocation;
use necsim_core::simulation::partial::event_sampler::PartialSimulation;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalEventSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
>(PhantomData<(H, G, D, R, S, C)>);

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > Default for UnconditionalEventSampler<H, G, D, R, S, C>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, D, R, S, C)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > EventSampler<H, G, D, R, S, C> for UnconditionalEventSampler<H, G, D, R, S, C>
{
    #[must_use]
    fn sample_event_for_lineage_at_indexed_location_time(
        &self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, D, R, S, C>,
        rng: &mut G,
    ) -> Event<H, R> {
        use necsim_core::cogs::RngSampler;

        let event_type = if rng.sample_event(*simulation.speciation_probability_per_generation) {
            EventType::Speciation
        } else {
            let dispersal_origin = indexed_location;
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(dispersal_origin.location(), rng);

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    simulation.habitat,
                    simulation.lineage_store,
                    rng,
                );

            EventType::Dispersal {
                origin: dispersal_origin,
                coalescence: optional_coalescence,
                target: dispersal_target,
                _marker: PhantomData::<H>,
            }
        };

        Event::new(event_time, lineage_reference, event_type)
    }
}
