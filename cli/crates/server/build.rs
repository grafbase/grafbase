#![allow(clippy::panic)] // this is a build script, an explicit panic is more readable than Result

use std::{env, fs, io, path::Path, time};

/// Env var name
const GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH: &str = "GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH";
const ASSETS_GZIP_PATH: &str = "./assets/assets.tar.gz";

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

fn decompress_assets() -> io::Result<tempfile::TempDir> {
    use flate2::bufread::GzDecoder;
    let dir = tempfile::tempdir()?;
    let assets_gzip_path = env::var("GRAFBASE_ASSETS_GZIP_PATH").unwrap_or_else(|_| ASSETS_GZIP_PATH.to_owned());
    eprintln!("Decompressing the assets at `{assets_gzip_path}` from `server` crate build script.");
    let file_reader = io::BufReader::new(fs::File::open(assets_gzip_path)?);
    let assets_reader = GzDecoder::new(file_reader);
    tar::Archive::new(assets_reader).unpack(dir.path())?;
    Ok(dir)
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

fn main() -> io::Result<()> {
    let start = time::Instant::now();

    let tmp_assets = decompress_assets()?;
    eprintln!(
        "⏱️ Timing after decompress_assets(): {:?}",
        time::Instant::now().duration_since(start)
    );
    let bundle_location = find_pathfinder_bundle_location();
    eprintln!(
        "⏱️ Timing after find_pathfinder_bundle_location(): {:?}",
        time::Instant::now().duration_since(start)
    );

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

    let origin_path = Path::new("../../udf-wrapper");
    let dest_path = tmp_assets.path();
    fs::create_dir_all(&target_path)?;
    fs::copy(
        origin_path.join("bun-multi-wrapper.ts"),
        dest_path.join("custom-resolvers/bun-multi-wrapper.ts"),
    )?;
    let _ = fs::remove_file(dest_path.join("custom-resolvers/wrapper.js"));
    fs::metadata(origin_path.join("dist.js")).expect("Building the worker wrapper is required to continue. Please run `npm install && npm run build` in 'cli/udf-wrapper'");
    fs::copy(
        origin_path.join("dist.js"),
        dest_path.join("custom-resolvers/wrapper.js"),
    )?;
    recompress_assets(tmp_assets.path())?;
    eprintln!(
        "⏱️ Timing after recompress: {:?}",
        time::Instant::now().duration_since(start)
    );

    // Tell Cargo to rerun this script only if the assets or the pathfinder bundle changed.
    println!("cargo:rerun-if-changed={bundle_location}");
    println!("cargo:rerun-if-env-changed={GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH}");
    println!("cargo:rerun-if-changed={ASSETS_GZIP_PATH}");

    Ok(())
}
