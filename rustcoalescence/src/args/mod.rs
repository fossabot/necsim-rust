use std::path::PathBuf;

use derive_getters::Getters;
use structopt::StructOpt;

mod algorithm;
mod area;

pub use algorithm::Algorithm;
use area::try_parse_area;

#[derive(Debug, StructOpt, Getters)]
pub struct CommandLineArguments {
    #[structopt(subcommand)]
    command: Command,
    #[structopt(flatten)]
    common_args: CommonArgs,
}

#[derive(Debug, StructOpt, Getters)]
#[allow(clippy::module_name_repetitions)]
pub struct CommonArgs {
    #[structopt(long = "speciation")]
    speciation_probability_per_generation: f64,
    #[structopt(long = "sample")]
    sample_percentage: f64,
    #[structopt(long)]
    seed: u64,
    #[structopt(
        possible_values = &Algorithm::variants(),
        case_insensitive = true,
        long = "algorithm"
    )]
    algorithm: Algorithm,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    InMemory(InMemoryArgs),
    NonSpatial(NonSpatialArgs),
    AlmostInfinite(AlmostInfiniteArgs),
}

#[derive(Debug, StructOpt, Getters)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryArgs {
    #[structopt(parse(from_os_str))]
    habitat_map: PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: PathBuf,
    #[structopt(long)]
    strict_load: bool,
}

impl NonSpatialArgs {
    pub fn as_in_memory(&self) -> InMemoryArgs {
        InMemoryArgs {
            habitat_map: PathBuf::from(format!(
                "NonSpatial/{}/{}/{}/Habitat",
                self.area.0, self.area.1, self.deme
            )),
            dispersal_map: PathBuf::from(format!(
                "NonSpatial/{}/{}/{}/Dispersal",
                self.area.0, self.area.1, self.deme
            )),
            strict_load: true,
        }
    }
}

#[derive(Debug, StructOpt, Getters)]
#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialArgs {
    #[structopt(parse(try_from_str = try_parse_area))]
    area: (u32, u32),
    deme: u32,
    #[structopt(long)]
    spatial: bool,
}

#[derive(Debug, StructOpt, Getters)]
#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteArgs {
    radius: u32,
    sigma: f64,
}
