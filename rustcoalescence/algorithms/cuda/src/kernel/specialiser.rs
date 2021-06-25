use std::ffi::CStr;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rust_cuda::rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

extern "Rust" {
    fn get_ptx_cstr_for_specialisation(specialisation: &str) -> &'static CStr;
}

pub fn get_ptx_cstr<
    H: Habitat + RustToCuda,
    G: PrimeableRng + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: LineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, R, S> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    T: TurnoverRate<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry + RustToCuda,
    A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
>() -> &'static CStr {
    let type_name_cstring = type_name_of(
        get_ptx_cstr::<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    );

    unsafe { get_ptx_cstr_for_specialisation(type_name_cstring) }
}

fn type_name_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}
