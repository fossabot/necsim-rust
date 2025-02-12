use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{abort, exit, Command},
};

use ptx_builder::{
    builder::{BuildStatus, Builder},
    error::{BuildErrorKind, Error, Result},
    reporter::ErrorLogPrinter,
};

use tempfile::NamedTempFile;

use quote::quote;

const SIMULATION_SPECIALISATION_HINT: &str =
    "rustcoalescence_algorithms_cuda::kernel::specialiser::get_ptx_cstr";
const SIMULATION_SPECIALISATION_ENV: &str = "RUSTCOALESCENCE_CUDA_KERNEL_SPECIALISATION";

fn extract_specialisation(input: &str) -> Option<&str> {
    let mut depth = 0_i32;

    for (i, c) in input.char_indices() {
        if c == '<' {
            depth += 1
        } else if c == '>' {
            depth -= 1
        }

        if depth <= 0 {
            return Some(&input[..(i + c.len_utf8())]);
        }
    }

    None
}

fn build_kernel_with_specialisation(specialisation: &str) -> Result<PathBuf> {
    env::set_var(SIMULATION_SPECIALISATION_ENV, specialisation);

    match Builder::new("../algorithms/cuda/kernel")?.build()? {
        BuildStatus::Success(output) => {
            let ptx_path = output.get_assembly_path();

            let mut specialised_ptx_path = ptx_path.clone();
            specialised_ptx_path.set_extension(&format!(
                "{:016x}.ptx",
                seahash::hash(specialisation.as_bytes())
            ));

            fs::copy(&ptx_path, &specialised_ptx_path).map_err(|err| {
                Error::from(BuildErrorKind::BuildFailed(vec![format!(
                    "Failed to copy kernel from {:?} to {:?}: {}",
                    ptx_path, specialised_ptx_path, err,
                )]))
            })?;

            fs::OpenOptions::new()
                .append(true)
                .open(&specialised_ptx_path)
                .and_then(|mut file| writeln!(file, "\n// {}", specialisation))
                .map_err(|err| {
                    Error::from(BuildErrorKind::BuildFailed(vec![format!(
                        "Failed to write specialisation to {:?}: {}",
                        specialised_ptx_path, err,
                    )]))
                })?;

            Ok(specialised_ptx_path)
        },
        BuildStatus::NotNeeded => Err(Error::from(BuildErrorKind::BuildFailed(vec![format!(
            "Kernel build for specialisation `{}` was not needed.",
            &specialisation
        )]))),
    }
}

fn main() -> ! {
    let args: Vec<String> = env::args().collect();

    let object_file_paths: Vec<&Path> = args
        .iter()
        .map(Path::new)
        .filter(|path| path.is_file() && path.extension().unwrap_or_else(|| "".as_ref()) == "o")
        .collect();

    let mut specialisations: Vec<String> = Vec::new();

    for path in object_file_paths.iter() {
        let output = Command::new("strings")
            .arg(path)
            .output()
            .expect("Failed to execute `strings`.");

        let stdout =
            std::str::from_utf8(&output.stdout).expect("Invalid output from `strings` command.");

        for mut line in stdout.lines() {
            while let Some(pos) = line.find(SIMULATION_SPECIALISATION_HINT) {
                line = &line[(pos + SIMULATION_SPECIALISATION_HINT.len())..];

                if let Some(specialisation) = extract_specialisation(line) {
                    line = &line[specialisation.len()..];

                    specialisations.push(specialisation.to_owned());
                }
            }
        }
    }

    let optional_temp_obj_file = if !specialisations.is_empty() {
        specialisations.sort_unstable();
        specialisations.dedup();

        let mut specialised_kernels: Vec<String> = Vec::with_capacity(specialisations.len());

        for specialisation in &specialisations {
            match build_kernel_with_specialisation(specialisation) {
                Ok(kernel_path) => {
                    let mut file = fs::File::open(&kernel_path).unwrap_or_else(|_| {
                        panic!("Failed to open kernel file at {:?}.", &kernel_path)
                    });

                    let mut kernel_ptx = String::new();

                    file.read_to_string(&mut kernel_ptx).unwrap_or_else(|_| {
                        panic!("Failed to read kernel file at {:?}.", &kernel_path)
                    });

                    specialised_kernels.push(kernel_ptx);
                },
                Err(error) => {
                    eprintln!("{}", ErrorLogPrinter::print(error));
                    exit(1);
                },
            }
        }

        let kernel_indices = (0..specialised_kernels.len()).map(syn::Index::from);
        let number_kernels = syn::Index::from(specialised_kernels.len());

        let specialisations: Vec<String> = specialisations
            .into_iter()
            .map(|s| format!("{}{}", SIMULATION_SPECIALISATION_HINT, s))
            .collect();

        let kernel_lookup_c_source = quote! {
            char const* SIMULATION_KERNEL_PTX_CSTRS[#number_kernels] = {#(#specialised_kernels),*};

            char const* get_ptx_cstr_for_specialisation(char const* specialisation) {
                #(
                    if (strcmp(specialisation, #specialisations) == 0) {
                        return SIMULATION_KERNEL_PTX_CSTRS[#kernel_indices];
                    }
                )*
            }
        };

        let mut kernel_lookup_c_source_file =
            NamedTempFile::new().expect("Failed to create a NamedTempFile.");

        write!(
            kernel_lookup_c_source_file,
            "#include<string.h>\n{}",
            kernel_lookup_c_source
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failed to write to kernel lookup source file at {:?}.",
                kernel_lookup_c_source_file.path()
            )
        });

        let kernel_lookup_c_obj_file =
            NamedTempFile::new().expect("Failed to create a NamedTempFile.");

        Command::new("cc")
            .arg("-c")
            .arg("-xc")
            .arg("-fPIC")
            .arg("-o")
            .arg(kernel_lookup_c_obj_file.path())
            .arg(kernel_lookup_c_source_file.path())
            .status()
            .expect("Failed to execute `cc`.");

        kernel_lookup_c_source_file
            .close()
            .expect("Failed to close the NamedTempFile.");

        Some(kernel_lookup_c_obj_file)
    } else {
        None
    };

    let mut linker = Command::new("cc");
    linker.args(&args[1..]);

    if let Some(ref temp_obj_file) = optional_temp_obj_file {
        linker.arg(temp_obj_file.path());
    }

    let status = linker.status().expect("Failed to execute `cc`.");

    if let Some(temp_obj_file) = optional_temp_obj_file {
        temp_obj_file
            .close()
            .expect("Failed to close the NamedTempFile.");
    }

    match status.code() {
        Some(code) => exit(code),
        None => abort(),
    }
}
