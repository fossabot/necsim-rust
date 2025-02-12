use std::{cell::UnsafeCell, ffi::CString, marker::PhantomData, ops::Deref};

use anyhow::Result;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rust_cuda::{
    rustacuda::{function::Function, module::Module},
    rustacuda_core::DeviceCopy,
};

use rust_cuda::{common::RustToCuda, host::CudaDropWrapper};
use rustcoalescence_algorithms_cuda_kernel_ptx_jit::host::compiler::PtxJITCompiler;

use super::{specialiser, SimulationKernel};

impl<
        'k,
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
    > SimulationKernel<'k, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    pub fn with_kernel<Q, F>(ptx_jit: bool, inner: F) -> Result<Q>
    where
        for<'s> F: FnOnce(
            &'s mut SimulationKernel<
                's,
                H,
                G,
                R,
                S,
                X,
                D,
                C,
                T,
                N,
                E,
                I,
                A,
                ReportSpeciation,
                ReportDispersal,
            >,
        ) -> Result<Q>,
    {
        // Load the module PTX &CStr containing the kernel function
        let ptx_cstr = specialiser::get_ptx_cstr::<
            H,
            G,
            R,
            S,
            X,
            D,
            C,
            T,
            N,
            E,
            I,
            A,
            ReportSpeciation,
            ReportDispersal,
        >();

        // Initialise the PTX JIT compiler with the original PTX source string
        let mut compiler = PtxJITCompiler::new(ptx_cstr);

        // Compile the CUDA module
        #[allow(unused_mut)]
        let mut module =
            UnsafeCell::new(CudaDropWrapper::from(Module::load_from_string(ptx_cstr)?));

        // Load the kernel function from the module
        let mut entry_point =
            unsafe { &*module.get() }.get_function(&CString::new("simulate").unwrap())?;

        // Safety: the mut `module` is only safe because:
        //  - `entry_point` is always dropped before `module` replaced
        //  - neither are mutably changed internally, only replaced
        let mut kernel = SimulationKernel {
            compiler: &mut compiler,
            ptx_jit,
            module: unsafe { &mut *module.get() },
            entry_point: &mut entry_point,
            marker: PhantomData::<(
                H,
                G,
                R,
                S,
                X,
                D,
                C,
                T,
                N,
                E,
                I,
                A,
                ReportSpeciation,
                ReportDispersal,
            )>,
        };

        inner(&mut kernel)
    }

    pub fn function(&self) -> &Function {
        self.entry_point
    }
}

impl<
        'k,
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
    > Deref
    for SimulationKernel<'k, H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    type Target = Module;

    fn deref(&self) -> &Self::Target {
        self.module
    }
}
