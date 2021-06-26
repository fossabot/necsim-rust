use std::env;

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg={}", profile);
    }

    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_ref) {
        println!("cargo:rustc-cfg=clippy");
    }

    // TODO: doesn't seem to work
    if matches!(env::var("CARGO"), Ok(cargo) if cargo.ends_with("rls")) {
        println!("cargo:rustc-cfg=rls")
    }

    // check: emits metadata, build: emits link (default)

    // for (key, value) in env::vars() {
    //     println!("cargo:warning={}: {}", key, value);
    // }
}
