use crate::consts::{EPHEMERAL_PORT_RANGE, WORKER_DIR, WORKER_FOLDER_VERSION_FILE};
use crate::types::Assets;
use crate::{bridge, errors::DevServerError};
use common::environment::Environment;
use common::types::LocalAddressType;
use common::utils::find_available_port_in_range;
use std::{
    fs,
    process::Stdio,
    thread::{self, JoinHandle},
};
use tokio::process::Command;
use version_compare::Version;

/// starts a development server by unpacking any files needed by the gateway worker
/// and starting the miniflare cli in `user_grafbase_path` in [`Environment`]
///
/// # Errors
///
/// returns [`DevServerError::ReadVersion`] if the version file for the extracted worker files cannot be read
///
/// returns [`DevServerError::CreateDir`] if the `WORKER_DIR` cannot be created
///
/// returns [`DevServerError::WriteFile`] if a file cannot be written into `WORKER_DIR`
///
/// # Panics
///
/// The spawned server and miniflare thread can panic if either of the two inner spawned threads panic
#[must_use]
pub fn start(port: u16) -> JoinHandle<Result<(), DevServerError>> {
    thread::spawn(move || {
        export_embedded_files()?;

        // the bridge runs on an available port within the ephemeral port range which is also supplied to the worker,
        // making the port choice and availability transprent to the user
        let bridge_port = find_available_port_in_range(EPHEMERAL_PORT_RANGE, LocalAddressType::Localhost)
            .ok_or(DevServerError::AvailablePort)?;

        spawn_servers(port, bridge_port)
    })
}

#[tokio::main]
async fn spawn_servers(worker_port: u16, bridge_port: u16) -> Result<(), DevServerError> {
    trace!("spawining miniflare");

    let environment = Environment::get();

    let bridge_handle = tokio::spawn(async move { bridge::start(bridge_port).await });

    // TODO: bundle a specific version of miniflare and extract it
    Command::new("npx")
        .args(&[
            "--quiet",
            "miniflare",
            "--port",
            &worker_port.to_string(),
            "--wrangler-config",
            "wrangler.toml",
        ])
        .current_dir(&environment.user_dot_grafbase_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(DevServerError::MiniflareError)?
        .wait()
        .await
        .map_err(DevServerError::MiniflareError)?;

    bridge_handle.await??;

    Ok(())
}

fn export_embedded_files() -> Result<(), DevServerError> {
    let environment = Environment::get();

    let worker_path = environment.user_dot_grafbase_path.join(WORKER_DIR);

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

            let full_path = environment.user_dot_grafbase_path.join(path.as_ref());

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
