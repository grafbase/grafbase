use crate::errors::DevServerError;
use common::environment::Environment;
use rust_embed::RustEmbed;
use std::{
    fs, io,
    process::{Command, Output, Stdio},
    thread::{self, JoinHandle},
};
use version_compare::Version;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

const WORKER_DIR: &str = "worker";
const WORKER_FOLDER_VERSION_FILE: &str = "version.txt";

/// starts a development server by unpacking any files needed by the gateway worker
/// and starting the miniflare cli in user_grafbase_path in [Environment]
pub fn start(port: u16) -> Result<JoinHandle<Result<Output, io::Error>>, DevServerError> {
    export_embedded_files()?;

    trace!("spawining miniflare");

    let environment = Environment::get();

    Ok(thread::spawn(move || {
        Command::new("npx")
            .arg("--quiet")
            .arg("miniflare")
            .arg("--port")
            .arg(port.to_string())
            .arg("-c")
            .arg("wrangler.toml")
            .current_dir(&environment.user_grafbase_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    }))
}

fn export_embedded_files() -> Result<(), DevServerError> {
    let environment = Environment::get();

    let worker_path = environment.user_grafbase_path.join(WORKER_DIR);

    // CARGO_PKG_VERSION is guaranteed be valid semver
    let current_version = Version::from(env!("CARGO_PKG_VERSION")).unwrap();

    let worker_version_path = worker_path.join(WORKER_FOLDER_VERSION_FILE);

    let export_files = if worker_path.is_dir() {
        let worker_version = fs::read_to_string(&worker_version_path).map_err(|_| DevServerError::ReadVersion)?;

        // derived from CARGO_PKG_VERSION, guaranteed be valid semver
        current_version > Version::from(&worker_version).unwrap()
    } else {
        true
    };

    // TODO: add dependency for gateway or add as build dep
    if export_files {
        trace!("writing worker files");

        fs::create_dir_all(&worker_path).map_err(|_| DevServerError::CreateDir(worker_path.clone()))?;

        let mut write_results = Assets::iter().map(|path| {
            let file = Assets::get(path.as_ref());

            let full_path = environment.user_grafbase_path.join(path.as_ref());

            // must be Some(file) since we're iterating over existing paths
            let write_result = fs::write(&full_path, file.unwrap().data);

            (write_result, full_path)
        });

        if let Some((_, path)) = write_results.find(|(result, _)| result.is_err()) {
            let error_path_string = path.to_string_lossy().to_string();
            return Err(DevServerError::WriteFile(error_path_string));
        }

        if fs::write(&worker_version_path, current_version.as_str()).is_err() {
            let worker_version_path_string = worker_version_path.to_string_lossy().to_string();
            return Err(DevServerError::WriteFile(worker_version_path_string));
        };
    }

    Ok(())
}
