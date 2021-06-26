use anyhow::Result;

#[cfg(not(any(clippy, rls)))]
mod kernels {
    // BUGS:
    // - rlib without hash is only output if the cli is built as a target
    // - in release mode, most strings are missing from the rlib, i.e. not all
    //   kernels are compiled

    #[cfg(debug)]
    rustcoalescence_derive::link_cuda_ptx_kernels! {
        kernel: "../algorithms/cuda/kernel",
        hint: "rustcoalescence_algorithms_cuda::kernel::specialiser::get_ptx_cstr",
        rlib: "../../target/debug/librustcoalescence_cli.rlib",
        env: "RUSTCOALESCENCE_CUDA_KERNEL_SPECIALISATION",
    }
    #[cfg(release)]
    rustcoalescence_derive::link_cuda_ptx_kernels! {
        kernel: "../algorithms/cuda/kernel",
        hint: "rustcoalescence_algorithms_cuda::kernel::specialiser::get_ptx_cstr",
        rlib: "../../target/release/librustcoalescence_cli.rlib",
        env: "RUSTCOALESCENCE_CUDA_KERNEL_SPECIALISATION",
    }
}

fn main() -> Result<()> {
    rustcoalescence_cli::cli()
}
