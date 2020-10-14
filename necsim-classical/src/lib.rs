#![deny(clippy::pedantic)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use array2d::Array2D;

use necsim_core::reporter::Reporter;
use necsim_core::rng::Rng;
use necsim_core::{simulation::Simulation, simulation::SimulationSettings};
use necsim_impls::event_generator::unconditional_global_lineage_store::GlobalLineageStoreUnconditionalEventGenerator;
use necsim_impls::landscape::in_memory_habitat_in_memory_precalculated_dispersal::LandscapeInMemoryHabitatInMemoryPrecalculatedDispersal;

pub struct ClassicalSimulation(std::marker::PhantomData<Simulation>);

impl ClassicalSimulation {
    /// Simulated the classical coalescence algorithm on an in memory
    /// `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    pub fn simulate(
        habitat: Array2D<u32>,
        habitat_map: &PathBuf,
        dispersal: &Array2D<f64>,
        dispersal_map: &PathBuf,
        speciation_probability_per_generation: f64,
        rng: &mut impl Rng,
        reporter: &mut impl Reporter,
    ) -> Result<(f64, usize)> {
        let landscape =
            LandscapeInMemoryHabitatInMemoryPrecalculatedDispersal::new(habitat, &dispersal)
                .with_context(|| {
                    format!(
                        concat!(
                            "Failed to create a Landscape with the habitat ",
                            "map {:?} and the dispersal map {:?}."
                        ),
                        dispersal_map, habitat_map
                    )
                })?;

        let settings = SimulationSettings::new(speciation_probability_per_generation, landscape);

        let (time, steps) = Simulation::simulate(
            &settings,
            GlobalLineageStoreUnconditionalEventGenerator::new(settings.landscape()),
            rng,
            reporter,
        );

        Ok((time, steps))
    }
}
