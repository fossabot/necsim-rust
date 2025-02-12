use core::{fmt, marker::PhantomData};

#[cfg(not(target_os = "cuda"))]
use rust_cuda::rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

use rust_cuda::utils::exchange::buffer::CudaExchangeBuffer;

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    reporter::{boolean::Boolean, Reporter},
};

#[cfg(target_os = "cuda")]
use necsim_core::impl_report;

#[allow(clippy::module_name_repetitions)]
#[derive(rust_cuda::common::RustToCuda, rust_cuda::host::LendToCuda)]
pub struct EventBuffer<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[r2cEmbed]
    speciation_buffer: CudaExchangeBuffer<Option<SpeciationEvent>>,
    #[r2cEmbed]
    dispersal_buffer: CudaExchangeBuffer<Option<DispersalEvent>>,
    max_events: usize,
    event_counter: usize,
    marker: PhantomData<(ReportSpeciation, ReportDispersal)>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> fmt::Debug
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("EventBuffer")
            .field("max_events", &self.max_events)
            .field("event_counter", &self.event_counter)
            .finish()
    }
}

#[cfg(not(target_os = "cuda"))]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
        let max_events = if ReportDispersal::VALUE {
            max_events
        } else if ReportSpeciation::VALUE {
            1_usize
        } else {
            0_usize
        };

        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;

        let speciation_capacity = if ReportSpeciation::VALUE {
            block_size * grid_size
        } else {
            1_usize // Caching space used to eliminate local memory usage
        };
        let dispersal_capacity = if ReportDispersal::VALUE {
            max_events * block_size * grid_size
        } else {
            1_usize // Caching space used to eliminate local memory usage
        };

        Ok(Self {
            speciation_buffer: CudaExchangeBuffer::new(&None, speciation_capacity)?,
            dispersal_buffer: CudaExchangeBuffer::new(&None, dispersal_capacity)?,
            max_events,
            event_counter: 0_usize,
            marker: PhantomData::<(ReportSpeciation, ReportDispersal)>,
        })
    }

    pub fn report_events<P>(&mut self, reporter: &mut P)
    where
        P: Reporter<ReportSpeciation = ReportSpeciation, ReportDispersal = ReportDispersal>,
    {
        for event in self.dispersal_buffer.iter_mut().filter_map(Option::take) {
            if ReportDispersal::VALUE {
                reporter.report_dispersal(&event.into());
            }
        }

        for event in self.speciation_buffer.iter_mut().filter_map(Option::take) {
            if ReportSpeciation::VALUE {
                reporter.report_speciation(&event.into());
            }
        }
    }
}

#[cfg(target_os = "cuda")]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Reporter
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    impl_report!(
        #[debug_requires(
            self.speciation_buffer[rust_cuda::device::utils::index()].is_none(),
            "does not report extraneous speciation event"
        )]
        speciation(&mut self, event: Used) {
            if ReportSpeciation::VALUE {
                self.speciation_buffer[rust_cuda::device::utils::index()] = Some(event.clone());
            } else {
                // Note: Using this cache avoids the use of local storage
                self.speciation_buffer[0] = Some(event.clone());
            }
        }
    );

    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous dispersal events"
        )]
        dispersal(&mut self, event: Used) {
            if ReportDispersal::VALUE {
                self.dispersal_buffer[rust_cuda::device::utils::index() * self.max_events + self.event_counter] = Some(event.clone());

                self.event_counter += 1;
            } else {
                // Note: Using this cache avoids the use of local storage
                self.dispersal_buffer[0] = Some(event.clone());
            }
        }
    );

    impl_report!(progress(&mut self, _progress: Ignored) {});
}
