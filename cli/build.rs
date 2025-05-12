#![allow(clippy::panic)]

use std::fs::exists;

fn main() {
    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());
    println!("cargo:rustc-env=DEBUG={}", std::env::var("DEBUG").unwrap());

    cynic_codegen::register_schema("grafbase")
        .from_sdl_file("src/api/graphql/api.graphql")
        .unwrap()
        .as_default()
        .unwrap();

    println!("cargo::rerun-if-changed=assets/cli-app.tar.gz");

    if !exists("assets/cli-app.tar.gz").is_ok_and(|exists| exists) {
        panic!(
            "The CLI dev app assets are missing and are required to build the CLI, please run `cargo make fetch-assets` in the `cli` directory"
        );
    }
}
