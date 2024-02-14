use common::environment::Environment;
use common::types::UdfKind;

use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use crate::consts::{ENTRYPOINT_SCRIPT_FILE_NAME, KV_FILE_NAME};
use crate::errors::{JavascriptPackageManagerComamndError, ServerError, UdfBuildError};

async fn run_command<P: AsRef<Path>>(
    command_type: JavaScriptPackageManager,
    arguments: &[&str],
    current_directory: P,
    tracing: bool,
    environment: &[(&'static str, &str)],
) -> Result<Option<Vec<u8>>, JavascriptPackageManagerComamndError> {
    let command_string = format!("{command_type} {}", arguments.iter().format(" "));
    let current_directory = current_directory.as_ref();
    match current_directory.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(JavascriptPackageManagerComamndError::WorkingDirectoryNotFound(
            current_directory.to_owned(),
        )),
        Err(err) => Err(JavascriptPackageManagerComamndError::WorkingDirectoryCannotBeRead(
            current_directory.to_owned(),
            err,
        )),
    }?;
    trace!("running '{command_string}'");

    // Use `which` to work-around weird path search issues on Windows.
    // See https://github.com/rust-lang/rust/issues/37519.
    let program_path = which::which(command_type.to_string())
        .map_err(|err| JavascriptPackageManagerComamndError::NotFound(command_type, err))?;

    let mut command = Command::new(program_path);
    command
        .envs(environment.iter().copied())
        .args(arguments)
        .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .current_dir(current_directory);

    trace!("Spawning {command:?}");
    let command = command
        .spawn()
        .map_err(|err| JavascriptPackageManagerComamndError::CommandError(command_type, err))?;

    let output = command
        .wait_with_output()
        .await
        .map_err(|err| JavascriptPackageManagerComamndError::CommandError(command_type, err))?;

    if output.status.success() {
        trace!("'{command_string}' succeeded");
        Ok(Some(output.stdout).filter(|output| !output.is_empty()))
    } else {
        trace!("'{command_string}' failed");
        Err(JavascriptPackageManagerComamndError::OutputError(
            command_type,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

#[derive(Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum JavaScriptPackageManager {
    Npm,
    Pnpm,
    Yarn,
}

pub(crate) const LOCK_FILE_NAMES: &[(&str, JavaScriptPackageManager)] = &[
    ("package-lock.json", JavaScriptPackageManager::Npm),
    ("pnpm-lock.yaml", JavaScriptPackageManager::Pnpm),
    ("yarn.lock", JavaScriptPackageManager::Yarn),
];

async fn extract_udf_wrapper_worker_contents(udf_kind: UdfKind) -> Result<String, UdfBuildError> {
    trace!("extracting {udf_kind} wrapper worker contents");
    let environment = Environment::get();
    tokio::fs::read_to_string(
        environment
            .user_dot_grafbase_path
            .join(crate::consts::WRAPPER_WORKER_JS_PATH),
    )
    .await
    .map_err(|err| UdfBuildError::ExtractUdfWrapperWorkerContents(udf_kind, err))
}

const UDF_EXTENSIONS: [&str; 2] = ["js", "ts"];

#[allow(clippy::too_many_lines)]
pub(crate) async fn build(
    environment: &Environment,
    environment_variables: &std::collections::HashMap<String, String>,
    udf_kind: UdfKind,
    udf_name: &str,
    _tracing: bool,
) -> Result<PathBuf, UdfBuildError> {
    use path_slash::PathBufExt as _;

    let project = environment.project.as_ref().expect("must be present");

    // FIXME: that's a hack, need to change the wrapper script to only check the final part of the
    // URL.
    let udf_wrapper_worker_contents = extract_udf_wrapper_worker_contents(udf_kind)
        .await?
        .replace("\"/invoke\"", &format!("\"{}\"", udf_url_path(udf_kind, udf_name)));

    trace!("building {udf_kind} '{udf_name}'");

    let udf_input_file_path_without_extension = project.udfs_source_path(udf_kind).join(udf_name);
    let udf_build_artifact_directory_path = project.udfs_build_artifact_path(udf_kind).join(udf_name);

    let mut udf_input_file_path = None;
    for extension in UDF_EXTENSIONS {
        let possible_udf_input_file_path = udf_input_file_path_without_extension.with_extension(extension);
        if tokio::fs::try_exists(&possible_udf_input_file_path)
            .await
            .expect("must succeed")
        {
            udf_input_file_path = Some(possible_udf_input_file_path);
        }
    }
    let udf_input_file_path = udf_input_file_path
        .ok_or_else(|| UdfBuildError::UdfDoesNotExist(udf_kind, udf_input_file_path_without_extension.clone()))?;

    let udf_wrapper_worker_contents = udf_wrapper_worker_contents
        .replace(
            "${UDF_MAIN_FILE_PATH}",
            udf_input_file_path.to_slash().expect("must be valid UTF-8").as_ref(),
        )
        .replace(
            "${UDF_KV_FILE_PATH}",
            project
                .dot_grafbase_directory_path
                .join(KV_FILE_NAME)
                .to_slash()
                .expect("must be valid UTF-8")
                .as_ref(),
        );

    tokio::fs::create_dir_all(&udf_build_artifact_directory_path)
        .await
        .map_err(|_err| UdfBuildError::CreateDir(udf_build_artifact_directory_path.clone(), udf_kind))?;

    let udf_build_package_json_path = udf_build_artifact_directory_path.join("package.json");

    let package_json = serde_json::json!({
        "main": ENTRYPOINT_SCRIPT_FILE_NAME,
    });
    let new_package_json_contents = serde_json::to_string_pretty(&package_json).expect("must be valid JSON");
    trace!("new package.json contents:\n{new_package_json_contents}");

    tokio::fs::write(&udf_build_package_json_path, new_package_json_contents)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_build_package_json_path.clone(), udf_kind, err))?;

    let dist_path = udf_build_package_json_path
        .parent()
        .expect("must have parent")
        .join("dist");

    tokio::fs::create_dir_all(&dist_path)
        .await
        .map_err(|_| UdfBuildError::CreateDir(dist_path.clone(), udf_kind))?;

    let process_env_prelude = format!(
        "globalThis.process.env = {};",
        serde_json::to_string(&environment_variables).expect("must be valid JSON")
    );

    let content_with_env = format!("{process_env_prelude}\n{udf_wrapper_worker_contents}");

    let entrypoint_js_path = dist_path.join(ENTRYPOINT_SCRIPT_FILE_NAME);

    tokio::fs::write(&entrypoint_js_path, content_with_env)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(entrypoint_js_path.clone(), udf_kind, err))?;

    Ok(udf_build_package_json_path)
}

pub(crate) fn udf_url_path(kind: UdfKind, name: &str) -> String {
    format!("/{}/{}/invoke", kind.to_string().to_lowercase(), slug::slugify(name))
}

const BUN_VERSION: &str = "1.0.26";

async fn installed_bun_version(bun_installation_path: impl AsRef<Path>) -> Option<String> {
    let bun_installation_path = bun_installation_path.as_ref();
    let bun_arguments = &[
        "exec",
        "--no",
        "--prefix",
        bun_installation_path.to_str().expect("must be valid"),
        "--",
        "bun",
        "--version",
    ];
    let output_bytes = run_command(
        JavaScriptPackageManager::Npm,
        bun_arguments,
        bun_installation_path,
        false,
        &[],
    )
    .await
    .ok()??;
    Some(String::from_utf8(output_bytes).ok()?.trim().to_owned())
}

pub(crate) async fn install_bun(environment: &Environment, tracing: bool) -> Result<(), ServerError> {
    let lock_file_path = environment.user_dot_grafbase_path.join(".bun.install.lock");
    let mut lock_file = tokio::task::spawn_blocking(move || {
        let mut file = fslock::LockFile::open(&lock_file_path)?;
        file.lock()?;
        Ok(file)
    })
    .await?
    .map_err(ServerError::Lock)?;

    if let Some(installed_bun_version) = installed_bun_version(&environment.bun_installation_path).await {
        info!("Installed bun version: {installed_bun_version}");
        if installed_bun_version == BUN_VERSION {
            info!("bun of the desired version already installed, skipping…");
            return Ok(());
        }
    }

    let bun_installation_path_str = environment.bun_installation_path.to_str().expect("must be valid");

    info!("Installing bun…");
    tokio::fs::create_dir_all(&environment.bun_installation_path)
        .await
        .map_err(|_| ServerError::CreateDir(environment.bun_installation_path.clone()))?;
    // Install bun once and for all.
    run_command(
        JavaScriptPackageManager::Npm,
        &[
            "add",
            "--save-dev",
            &format!("bun@{BUN_VERSION}"),
            "--prefix",
            bun_installation_path_str,
        ],
        bun_installation_path_str,
        tracing,
        &[],
    )
    .await?;

    tokio::task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(ServerError::Unlock)?;

    Ok(())
}
