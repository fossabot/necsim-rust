use std::path::Path;

use anyhow::{Context, Result};
use array2d::Array2D;

use necsim_core_bond::NonNegativeF64;

use crate::args::{InMemoryTurnover, MapLoadingMode};

pub fn load_dispersal_map(
    path: &Path,
    loading_mode: MapLoadingMode,
) -> Result<Array2D<NonNegativeF64>> {
    (|| {
        let map = crate::tiff::load_map_from_tiff::<f64>(
            path,
            match loading_mode {
                MapLoadingMode::FixMe | MapLoadingMode::OffByOne => false,
                MapLoadingMode::Strict => true,
            },
        )?;

        if map.elements_row_major_iter().any(|x| *x < 0.0_f64) {
            anyhow::bail!("Dispersal weights are not all non-negative")
        }

        Ok(unsafe { std::mem::transmute(map) })
    })()
    .with_context(|| format!("Failed to load the dispersal map from {:?}.", path))
}

pub fn load_turnover_map(
    path: &Path,
    loading_mode: MapLoadingMode,
) -> Result<Array2D<NonNegativeF64>> {
    (|| {
        let map = crate::tiff::load_map_from_tiff::<f64>(
            path,
            match loading_mode {
                MapLoadingMode::FixMe | MapLoadingMode::OffByOne => false,
                MapLoadingMode::Strict => true,
            },
        )?;

        if map.elements_row_major_iter().any(|x| *x < 0.0_f64) {
            anyhow::bail!("Turnover rates are not all non-negative")
        }

        Ok(unsafe { std::mem::transmute(map) })
    })()
    .with_context(|| format!("Failed to load the turnover map from {:?}.", path))
}

pub fn load_habitat_map(
    path: &Path,
    turnover: &InMemoryTurnover,
    dispersal: &mut Array2D<NonNegativeF64>,
    loading_mode: MapLoadingMode,
) -> Result<Array2D<u32>> {
    let mut habitat = crate::tiff::load_map_from_tiff::<u32>(
        path,
        match loading_mode {
            MapLoadingMode::FixMe | MapLoadingMode::OffByOne => false,
            MapLoadingMode::Strict => true,
        },
    )
    .with_context(|| format!("Failed to load the habiat map from {:?}.", path))?;

    match loading_mode {
        MapLoadingMode::FixMe => {
            fix_habitat_map(&mut habitat, turnover, dispersal);
            fix_no_turnover_habitat_map(&mut habitat, turnover);
            fix_dispersal_map(&habitat, dispersal);
        },
        MapLoadingMode::OffByOne => fix_habitat_map(&mut habitat, turnover, dispersal),
        MapLoadingMode::Strict => (),
    };

    Ok(habitat)
}

// Fix habitat rounding error by correcting 0/1 values to 0/1 based on dispersal
//  (can only disperse from habitat) and turnover (no turnover -> no habitat)
fn fix_habitat_map(
    habitat: &mut Array2D<u32>,
    turnover: &InMemoryTurnover,
    dispersal: &Array2D<NonNegativeF64>,
) {
    for y in 0..habitat.num_rows() {
        for x in 0..habitat.num_columns() {
            let h_before = habitat[(y, x)];

            if h_before <= 1 {
                // If there is no turnover, there cannot be habitat at this location
                // If there is any dispersal from this location, it must be habitat
                let h_fixed = if match turnover {
                    InMemoryTurnover::Map(turnover) => turnover[(y, x)] == 0.0_f64,
                    InMemoryTurnover::Uniform(_) => false,
                } {
                    0
                } else if dispersal
                    .row_iter(y * habitat.num_columns() + x)
                    .map_or(false, |mut it| it.any(|p| *p > 0.0_f64))
                {
                    1
                } else {
                    0
                };

                if h_fixed != h_before {
                    habitat[(y, x)] = h_fixed;

                    // warn!(
                    //     "Corrected habitat value {} to {} at ({},{}) to fit \
                    //      turnover and dispersal ...", h_before, h_fixed, x, y
                    // );
                }
            }
        }
    }
}

// Sets habitat to 0 if there is no turnover
fn fix_no_turnover_habitat_map(habitat: &mut Array2D<u32>, turnover: &InMemoryTurnover) {
    if let InMemoryTurnover::Map(turnover) = turnover {
        for y in 0..habitat.num_rows() {
            for x in 0..habitat.num_columns() {
                if turnover[(y, x)] == 0.0_f64 {
                    habitat[(y, x)] = 0;

                    // warn!(
                    //     "Corrected habitat value {} to {} at ({},{}) to fit \
                    //      zero turnover ...", h_before, h_fixed, x, y
                    // );
                }
            }
        }
    }
}

// Fix dispersal by removing dispersal to/from non-habitat
// Fix dispersal by adding self-dispersal when no dispersal exists from habitat
fn fix_dispersal_map(habitat: &Array2D<u32>, dispersal: &mut Array2D<NonNegativeF64>) {
    let size = habitat.num_rows() * habitat.num_columns();

    for y in 0..habitat.num_rows() {
        for x in 0..habitat.num_columns() {
            let row = y * habitat.num_columns() + x;

            if habitat[(y, x)] == 0 {
                let mut warn = false;

                // Fix dispersal from non-habitat
                for column in 0..size {
                    let d = &mut dispersal[(row, column)];

                    if *d > 0.0_f64 {
                        warn = true;

                        *d = NonNegativeF64::zero();
                    }
                }

                if warn {
                    // warn!(
                    //     "Corrected dispersal values to 0.0 at ({},{}) to fit
                    // \      habitat ...", x, y,
                    // );
                }
            } else {
                let mut total_dispersal = NonNegativeF64::zero();

                // Fix dispersal to non-habitat
                for column in 0..size {
                    let tx = column % habitat.num_columns();
                    let ty = column / habitat.num_columns();

                    let d = &mut dispersal[(row, column)];

                    if habitat[(ty, tx)] == 0 && *d > 0.0_f64 {
                        // warn!(
                        //     "Corrected dispersal value {} to 0.0 at ({},{})->({},{}) to fit \
                        //      habitat ...",
                        //     *d, x, y, tx, ty,
                        // );

                        *d = NonNegativeF64::zero();
                    }

                    total_dispersal += *d;
                }

                // Fix no dispersal from habitat with self-dispersal
                if total_dispersal <= 0.0_f64 {
                    dispersal[(row, row)] = NonNegativeF64::one();
                }
            }
        }
    }
}
