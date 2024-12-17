#![allow(clippy::panic)]

use std::fs::exists;

fn main() {
    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());

    cynic_codegen::register_schema("grafbase")
        .from_sdl_file("src/backend/api/graphql/api.graphql")
        .unwrap()
        .as_default()
        .unwrap();

    println!("cargo::rerun-if-changed=assets/pathfinder.tar.gz");

    if !exists("assets/pathfinder.tar.gz").is_ok_and(|exists| exists) {
        panic!("The Pathfinder assets are missing and are required to build the CLI, please run `cargo make fetch-assets` in the `cli` directory");
    }
}
