use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};

use necsim_core::{
    event::{PackedEvent, TypedEvent},
    impl_report,
    reporter::Reporter,
};

use necsim_partitioning_core::LocalPartition;

pub struct BufferingReporterProxy<'p, R: Reporter, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    event_buffer: Vec<PackedEvent>,
    _marker: PhantomData<R>,
}

impl<'p, R: Reporter, P: LocalPartition<R>> fmt::Debug for BufferingReporterProxy<'p, R, P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct EventBufferLen(usize);

        impl fmt::Debug for EventBufferLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<PackedEvent; {}>", self.0)
            }
        }

        fmt.debug_struct("PartitionReporterProxy")
            .field("event_buffer", &EventBufferLen(self.event_buffer.len()))
            .finish()
    }
}

impl<'p, R: Reporter, P: LocalPartition<R>> Reporter for BufferingReporterProxy<'p, R, P> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportSpeciation>) {
        self.event_buffer.push(speciation.clone().into());
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportDispersal>) {
        self.event_buffer.push(dispersal.clone().into());
    });

    impl_report!(progress(&mut self, progress: MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportProgress>) {
        self.local_partition.get_reporter().report_progress(progress.into());
    });
}

impl<'p, R: Reporter, P: LocalPartition<R>> BufferingReporterProxy<'p, R, P> {
    pub fn from(local_partition: &'p mut P) -> Self {
        Self {
            local_partition,
            event_buffer: Vec::new(),
            _marker: PhantomData::<R>,
        }
    }

    pub fn clear_events(&mut self) {
        self.event_buffer.clear();
    }

    pub fn report_events(&mut self) {
        for event in self.event_buffer.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(&event.into());
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(&event.into());
                },
            }
        }
    }

    pub fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}
