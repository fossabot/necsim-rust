use std::env;

fn main() {
    match env::var("CARGO_CFG_FEATURE") {
        Ok(feature) if feature == "cargo-clippy" => println!("cargo:rustc-cfg=clippy"),
        _ => (),
    }

    // TODO: doesn't seem to work
    match env::var("CARGO") {
        Ok(cargo) if cargo.ends_with("rls") => println!("cargo:rustc-cfg=rls"),
        _ => (),
    }

    // check: emits metadata, build: emits link (default)

    // for (key, value) in env::vars() {
    //     println!("cargo:warning={}: {}", key, value);
    // }
}
