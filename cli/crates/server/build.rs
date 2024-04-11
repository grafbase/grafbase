#![allow(clippy::panic)] // this is a build script, an explicit panic is more readable than Result

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time,
};

/// Env var name
const GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH: &str = "GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH";

/// The path to the contents of the dist/ folder of a successful build of cli-app.
fn find_pathfinder_bundle_location() -> String {
    eprintln!("Making sure the pathfinder app is built...");
    if let Ok(location) = env::var(GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH) {
        assert!(
            fs::metadata(&location).is_ok(),
            "The location specified in {GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH} ({location} does not exist."
        );
        return location;
    }

    let default_path = "../../../packages/cli-app/dist";

    eprintln!("Using the default location for the pathfinder app...");
    if fs::metadata(default_path).is_ok() {
        return default_path.to_owned();
    }

    panic!(
        r#"
Could not find a bundled pathfinder to include in cli binary.

You must either provide the GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH environment variable or
build the Pathfinder app at the default location.

Please see the instructions in cli/crates/server/README.md.
    "#
    );
}

fn recompress_assets(assets_path: &Path) -> io::Result<()> {
    use flate2::{write::GzEncoder, Compression};

    eprintln!("Reconstructing assets.tar.gz");
    let out_file_path = Path::new(&env::var("OUT_DIR").unwrap()).join("assets.tar.gz");
    eprintln!("   Creating {out_file_path:?}");
    let out_file = fs::File::create(out_file_path)?;
    let out = GzEncoder::new(out_file, Compression::default());
    let mut tar_builder = tar::Builder::new(out);
    tar_builder.mode(tar::HeaderMode::Deterministic);
    eprintln!("   Adding {assets_path:?} to archive");
    tar_builder.append_dir_all(".", assets_path).unwrap();
    tar_builder.finish()?;

    Ok(())
}

fn copy_wrappers(dest_path: &Path) -> io::Result<()> {
    let origin_path = if let Ok(path) = env::var("GRAFBASE_CLI_WRAPPERS_PATH") {
        PathBuf::from(path)
    } else {
        Path::new("../../wrappers").to_owned()
    };
    fs::create_dir_all(dest_path.join("custom-resolvers"))?;
    fs::create_dir_all(dest_path.join("parser"))?;
    fs::copy(
        origin_path.join("bun-multi-wrapper.ts"),
        dest_path.join("custom-resolvers/bun-multi-wrapper.ts"),
    )?;
    fs::copy(
        origin_path.join("parse-config.ts"),
        dest_path.join("parser/parse-config.ts"),
    )?;
    fs::copy(
        origin_path.join("parse-config.mts"),
        dest_path.join("parser/parse-config.mts"),
    )?;
    fs::write(dest_path.join("package.json"), r#"{ "name": "assets" }"#)?;
    fs::metadata(origin_path.join("dist.js")).expect("Building the worker wrapper is required to continue. Please run `bun install && bun run build` in 'cli/wrappers'");
    fs::copy(
        origin_path.join("dist.js"),
        dest_path.join("custom-resolvers/wrapper.js"),
    )?;

    Ok(())
}

fn main() -> io::Result<()> {
    let start = time::Instant::now();

    let bundle_location = find_pathfinder_bundle_location();
    eprintln!(
        "⏱️ Timing after find_pathfinder_bundle_location(): {:?}",
        time::Instant::now().duration_since(start)
    );

    let tmp_assets = tempfile::tempdir()?;

    eprintln!("Copying bundled pathfinder to the assets dir...");
    let target_path = tmp_assets.path().join("static/assets");
    fs::create_dir_all(&target_path)?;
    for file in fs::read_dir(Path::new(&bundle_location).join("assets"))? {
        let file_path = file?.path();
        let dest_path = target_path.join(file_path.file_name().unwrap());
        eprintln!("    {file_path:?} -> {dest_path:?}");
        fs::copy(file_path, &dest_path)?;
    }
    eprintln!("⏱️ Timing after copy: {:?}", time::Instant::now().duration_since(start));

    eprintln!("Copying wrappers to the assets dir...");
    copy_wrappers(tmp_assets.path())?;

    recompress_assets(tmp_assets.path())?;
    eprintln!(
        "⏱️ Timing after recompress: {:?}",
        time::Instant::now().duration_since(start)
    );

    // Tell Cargo to rerun this script only if the assets or the pathfinder bundle changed.
    println!("cargo:rerun-if-changed={bundle_location}");
    println!("cargo:rerun-if-env-changed={GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH}");
    println!("cargo:rerun-if-changed=../../wrappers");

    Ok(())
}
