use common::environment::Environment;
use common::types::UdfKind;
use std::io;

use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

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
    tracing: bool,
    enable_kv: bool,
) -> Result<(PathBuf, PathBuf), UdfBuildError> {
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

    tokio::fs::create_dir_all(&udf_build_artifact_directory_path)
        .await
        .map_err(|_err| UdfBuildError::CreateDir(udf_build_artifact_directory_path.clone(), udf_kind))?;
    let udf_build_entrypoint_path = udf_build_artifact_directory_path.join("entrypoint.js");

    let udf_build_package_json_path = udf_build_artifact_directory_path.join("package.json");

    let artifact_directory_modules_path = udf_build_artifact_directory_path.join("node_modules");
    let artifact_directory_modules_path_string = artifact_directory_modules_path
        .to_str()
        .expect("must be valid if `artifact_directory_path_string` is valid");

    let udf_wrapper_worker_contents = udf_wrapper_worker_contents.replace(
        "${UDF_MAIN_FILE_PATH}",
        udf_input_file_path.to_slash().expect("must be valid UTF-8").as_ref(),
    );
    tokio::fs::write(&udf_build_entrypoint_path, udf_wrapper_worker_contents)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_build_entrypoint_path.clone(), udf_kind, err))?;

    let package_json = serde_json::json!({
        "module": "entrypoint.js",
        "type": "module"
    });
    let new_package_json_contents = serde_json::to_string_pretty(&package_json).expect("must be valid JSON");
    trace!("new package.json contents:\n{new_package_json_contents}");

    tokio::fs::write(&udf_build_package_json_path, new_package_json_contents)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_build_package_json_path.clone(), udf_kind, err))?;

    // FIXME
    let esbuild_arguments: &[&str] = &[];

    let _: Result<_, _> = tokio::fs::remove_file(&wrangler_toml_file_path).await;
    // FIXME ESBUILD HERE
    run_command(
        JavaScriptPackageManager::Npm,
        esbuild_arguments,
        &environment.esbuild_installation_path,
        tracing,
        &[],
    )
    .await
    .map_err(|err| match err {
        JavascriptPackageManagerComamndError::OutputError(_, output) => UdfBuildError::EsbuildBuildFailed { output },
        other => other.into(),
    })?;

    let process_env_prelude = format!(
        "globalThis.process = {{ env: {} }};",
        serde_json::to_string(&environment_variables).expect("must be valid JSON")
    );

    let (temp_file, temp_file_path) = tokio::task::spawn_blocking(tempfile::NamedTempFile::new)
        .await?
        .map_err(UdfBuildError::CreateTemporaryFile)?
        .into_parts();
    {
        let mut temp_file: tokio::fs::File = temp_file.into();
        temp_file
            .write_all(process_env_prelude.as_bytes())
            .await
            .map_err(|err| UdfBuildError::CreateNotWriteToTemporaryFile(temp_file_path.to_path_buf(), err))?;
        temp_file
            .write_all(
                // FIXME join with worker path
                &tokio::fs::read("entrypoint.js").await.expect("must succeed"),
            )
            .await
            .map_err(|err| UdfBuildError::CreateNotWriteToTemporaryFile(temp_file_path.to_path_buf(), err))?;
    }
    // FIXME join with worker path
    let entrypoint_js_path = "entrypoint.js";
    tokio::fs::copy(temp_file_path, &entrypoint_js_path)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(entrypoint_js_path.clone(), udf_kind, err))?;

    let slugified_udf_name = slug::slugify(udf_name);
    let udf_url_path = udf_url_path(udf_kind, udf_name);

    Ok((udf_build_package_json_path, wrangler_toml_file_path))
}

pub(crate) fn udf_url_path(kind: UdfKind, name: &str) -> String {
    format!("/{kind}/{}/invoke", slug::slugify(name))
}

const ESBUILD_VERSION: &str = "0.19.11";

async fn installed_esbuild_version(esbuild_installation_path: impl AsRef<Path>) -> Option<String> {
    let esbuild_installation_path = esbuild_installation_path.as_ref();
    let esbuild_arguments = &[
        "exec",
        "--no",
        "--prefix",
        esbuild_installation_path.to_str().expect("must be valid"),
        "--",
        "esbuild",
        "--version",
    ];
    let output_bytes = run_command(
        JavaScriptPackageManager::Npm,
        esbuild_arguments,
        esbuild_installation_path,
        false,
        &[],
    )
    .await
    .ok()??;
    Some(String::from_utf8(output_bytes).ok()?.trim().to_owned())
}

// FIXME check if we can bundle ESBUILD
pub(crate) async fn install_esbuild(environment: &Environment, tracing: bool) -> Result<(), ServerError> {
    let lock_file_path = environment.user_dot_grafbase_path.join(".esbuild.install.lock");
    let mut lock_file = tokio::task::spawn_blocking(move || {
        let mut file = fslock::LockFile::open(&lock_file_path)?;
        file.lock()?;
        Ok(file)
    })
    .await?
    .map_err(ServerError::Lock)?;

    if let Some(installed_esbuild_version) = installed_esbuild_version(&environment.esbuild_installation_path).await {
        info!("Installed esbuild version: {installed_esbuild_version}");
        if installed_esbuild_version == ESBUILD_VERSION {
            info!("esbuild of the desired version already installed, skipping…");
            return Ok(());
        }
    }

    let esbuild_installation_path_str = environment.esbuild_installation_path.to_str().expect("must be valid");

    info!("Installing esbuild…");
    tokio::fs::create_dir_all(&environment.esbuild_installation_path)
        .await
        .map_err(|_| ServerError::CreateDir(environment.esbuild_installation_path.clone()))?;
    // Install esbuild once and for all.
    run_command(
        JavaScriptPackageManager::Npm,
        &[
            "add",
            "--save-dev",
            &format!("esbuild@{ESBUILD_VERSION}"),
            "--prefix",
            esbuild_installation_path_str,
        ],
        esbuild_installation_path_str,
        tracing,
        &[],
    )
    .await?;

    tokio::task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(ServerError::Unlock)?;

    Ok(())
}
