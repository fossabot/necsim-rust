use core::num::NonZeroU32;

use necsim_core::{
    cogs::{Backup, Habitat},
    landscape::Location,
};

pub mod equal;
pub mod modulo;
pub mod monolithic;
pub mod radial;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Decomposition<H: Habitat>: Backup + Sized + core::fmt::Debug {
    #[debug_ensures(
        ret < self.get_number_of_subdomains().get(),
        "subdomain rank is in range [0, self.get_number_of_subdomains())"
    )]
    fn get_subdomain_rank(&self) -> u32;

    fn get_number_of_subdomains(&self) -> NonZeroU32;

    #[debug_requires(
        habitat.contains(location),
        "location is contained inside habitat"
    )]
    #[debug_ensures(
        ret < self.get_number_of_subdomains().get(),
        "subdomain rank is in range [0, self.get_number_of_subdomains())"
    )]
    fn map_location_to_subdomain_rank(&self, location: &Location, habitat: &H) -> u32;
}
