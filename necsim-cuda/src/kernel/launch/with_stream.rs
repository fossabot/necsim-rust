use std::marker::PhantomData;

use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
    HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
};

use rustacuda::{
    function::{BlockSize, GridSize},
    stream::Stream,
};
use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

use super::SimulationKernelWithDimensions;

use rustacuda::function::Function;

#[allow(clippy::type_complexity)]
pub struct SimulationKernelWithDimensionsStream<
    'k,
    's,
    H: HabitatToU64Injection + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: EventSampler<H, G, D, R, S, C> + RustToCuda,
    A: ActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    pub(super) entry_point: &'k Function<'k>,
    pub(super) marker: PhantomData<(H, G, D, R, S, C, E, A)>,
    pub(super) grid_size: GridSize,
    pub(super) block_size: BlockSize,
    pub(super) shared_mem_bytes: u32,
    pub(super) stream: &'s Stream,
}

impl<
        'k,
        H: HabitatToU64Injection + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: EventSampler<H, G, D, R, S, C> + RustToCuda,
        A: ActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    >
    SimulationKernelWithDimensions<'k, H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    #[allow(clippy::type_complexity)]
    pub fn with_stream<'s>(
        &self,
        stream: &'s Stream,
    ) -> SimulationKernelWithDimensionsStream<
        'k,
        's,
        H,
        G,
        D,
        R,
        S,
        C,
        E,
        A,
        REPORT_SPECIATION,
        REPORT_DISPERSAL,
    > {
        SimulationKernelWithDimensionsStream {
            entry_point: self.entry_point,
            marker: self.marker,
            grid_size: self.grid_size.clone(),
            block_size: self.block_size.clone(),
            shared_mem_bytes: self.shared_mem_bytes,
            stream,
        }
    }
}
