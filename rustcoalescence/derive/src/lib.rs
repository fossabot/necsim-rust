use proc_macro::TokenStream;
use quote::quote;

use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{exit, Command},
};

use ptx_builder::{
    builder::{BuildStatus, Builder},
    error::{BuildErrorKind, Error, Result},
    reporter::ErrorLogPrinter,
};

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

fn build_kernel_with_specialisation(kernel_path: &Path, env_var: &str, specialisation: &str) -> Result<PathBuf> {
    env::set_var(env_var, specialisation);

    match Builder::new(kernel_path)?.build()? {
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

mod config {
    syn::custom_keyword!(kernel);
    syn::custom_keyword!(hint);
    syn::custom_keyword!(rlib);
    syn::custom_keyword!(env);
}

struct LinkerConfig {
    kernel: String,
    hint: String,
    rlib: String,
    env: String,
}

impl syn::parse::Parse for LinkerConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _kernel_ident: config::kernel = input.parse()?;
        let _kernel_column: syn::Token![:] = input.parse()?;
        let kernel: syn::LitStr = input.parse()?;
        let _kernel_comma: syn::Token![,] = input.parse()?;

        let _hint_ident: config::hint = input.parse()?;
        let _hint_column: syn::Token![:] = input.parse()?;
        let hint: syn::LitStr = input.parse()?;
        let _hint_comma: syn::Token![,] = input.parse()?;

        let _rlib_ident: config::rlib = input.parse()?;
        let _rlib_column: syn::Token![:] = input.parse()?;
        let rlib: syn::LitStr = input.parse()?;
        let _rlib_comma: syn::Token![,] = input.parse()?;

        let _env_ident: config::env = input.parse()?;
        let _env_column: syn::Token![:] = input.parse()?;
        let env: syn::LitStr = input.parse()?;
        let _env_comma: Option<syn::Token![,]> = input.parse()?;

        Ok(Self {
            kernel: kernel.value(),
            hint: hint.value(),
            rlib: rlib.value(),
            env: env.value(),
        })
    }
}

#[proc_macro]
pub fn link_cuda_ptx_kernels(tokens: TokenStream) -> TokenStream {
    let config = syn::parse_macro_input!(tokens as LinkerConfig);

    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let output = Command::new("strings")
        .arg(base_path.join(config.rlib))
        .output()
        .expect("Failed to execute `strings`.");

    let stdout =
        std::str::from_utf8(&output.stdout).expect("Invalid output from `strings` command.");

    let mut specialisations = Vec::new();

    for mut line in stdout.lines() {
        while let Some(pos) = line.find(&config.hint) {
            line = &line[(pos + config.hint.len())..];

            if let Some(specialisation) = extract_specialisation(line) {
                line = &line[specialisation.len()..];

                specialisations.push(specialisation.to_owned());
            }
        }
    }

    if specialisations.is_empty() {
        return quote!().into();
    }

    specialisations.sort_unstable();
    specialisations.dedup();

    let kernel_path = base_path.join(&config.kernel);

    let mut specialised_kernels: Vec<String> = Vec::with_capacity(specialisations.len());

    for specialisation in &specialisations {
        match build_kernel_with_specialisation(&kernel_path, &config.env, specialisation) {
            Ok(kernel_path) => {
                let mut file = fs::File::open(&kernel_path).unwrap_or_else(|_| {
                    panic!("Failed to open kernel file at {:?}.", &kernel_path)
                });

                let mut kernel_ptx = String::new();

                file.read_to_string(&mut kernel_ptx).unwrap_or_else(|_| {
                    panic!("Failed to read kernel file at {:?}.", &kernel_path)
                });

                kernel_ptx.push('\0');

                specialised_kernels.push(kernel_ptx);
            },
            Err(error) => {
                eprintln!("{}", ErrorLogPrinter::print(error));
                exit(1);
            },
        }
    }

    (quote!{
        #[no_mangle]
        extern "Rust" fn get_ptx_cstr_for_specialisation(specialisation: &str) -> &'static std::ffi::CStr {
            let ptx_str_with_nul = match specialisation {
                #(#specialisations => #specialised_kernels),*,
                _ => unreachable!("Unknown CUDA kernel specialisation"),
            };

            unsafe { std::ffi::CStr::from_ptr(ptx_str_with_nul.as_ptr().cast()) }
        }
    }).into()
}
