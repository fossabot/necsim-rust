use anyhow::Result;

#[cfg(not(any(clippy, rls)))]
rustcoalescence_derive::link_cuda_ptx_kernels!{
    kernel: "../algorithms/cuda/kernel",
    hint: "rustcoalescence_algorithms_cuda::kernel::specialiser::get_ptx_cstr",
    rlib: "../../target/debug/deps/librustcoalescence_cli-597bacfe911eb896.rlib",
    env: "RUSTCOALESCENCE_CUDA_KERNEL_SPECIALISATION",
}

fn main() -> Result<()> {
    rustcoalescence_cli::cli()
}
