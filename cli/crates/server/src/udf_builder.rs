use std::path::{Path, PathBuf};
use std::process::Stdio;

use common::environment::{Environment, Project};
use common::types::UdfKind;
use futures_util::pin_mut;
use itertools::Itertools;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::errors::{JavascriptPackageManagerComamndError, ServerError, UdfBuildError};

async fn run_command<P: AsRef<Path>>(
    command_type: JavaScriptPackageManager,
    arguments: &[&str],
    current_directory: P,
    tracing: bool,
    environment: &[(&'static str, &str)],
) -> Result<(), JavascriptPackageManagerComamndError> {
    let command_string = format!("{command_type} {}", arguments.iter().format(" "));
    let current_directory = current_directory.as_ref();
    if !current_directory.exists() {
        return Err(JavascriptPackageManagerComamndError::WorkingDirectoryNotFound(
            current_directory.to_owned(),
        ));
    }
    debug!("running '{command_string}'");

    // Use `which` to work-around weird path search issues on Windows.
    // See https://github.com/rust-lang/rust/issues/37519.
    let program_path = which::which(command_type.to_string())
        .map_err(|err| JavascriptPackageManagerComamndError::NotFound(command_type, err.to_string()))?;

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
        Ok(())
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

async fn guess_package_manager_from_package_json(path: impl AsRef<Path>) -> Option<JavaScriptPackageManager> {
    let path = path.as_ref();
    // FIXME: In the future, we may honour the version too.
    // "packageManager": "^pnpm@1.2.3"
    // "packageManager": "^yarn@2.3.4"
    // etc.
    let object = match serde_json::from_slice(&tokio::fs::read(&path).await.ok()?) {
        Ok(serde_json::Value::Object(object)) => object,
        other => {
            warn!("Invalid package.json contents: {other:?} in path {}.", path.display());
            return None;
        }
    };
    object
        .get("packageManager")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.trim_start_matches('^').split('@').next().unwrap().parse().ok())
}

pub const LOCK_FILES: &[(&str, JavaScriptPackageManager)] = &[
    ("package-lock.json", JavaScriptPackageManager::Npm),
    ("pnpm-lock.yaml", JavaScriptPackageManager::Pnpm),
    ("yarn.lock", JavaScriptPackageManager::Yarn),
];

async fn guess_package_manager_from_package_root(path: impl AsRef<Path>) -> Option<JavaScriptPackageManager> {
    let package_root = path.as_ref();

    futures_util::future::join_all(LOCK_FILES.iter().map(|(file_name, package_manager)| {
        let path_to_check = package_root.join(file_name);
        async move {
            let file_exists = tokio::fs::try_exists(&path_to_check).await.ok().unwrap_or_default();
            if file_exists {
                Some(*package_manager)
            } else {
                None
            }
        }
    }))
    .await
    .into_iter()
    .flatten()
    .next()
}

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

#[allow(clippy::too_many_lines)]
pub async fn build(
    environment: &Environment,
    project: &Project,
    environment_variables: &std::collections::HashMap<String, String>,
    udf_kind: UdfKind,
    udf_name: &str,
    tracing: bool,
) -> Result<(PathBuf, PathBuf), UdfBuildError> {
    use futures_util::StreamExt;
    use path_slash::PathBufExt as _;

    const EXTENSIONS: [&str; 2] = ["js", "ts"];

    let udf_wrapper_worker_contents = extract_udf_wrapper_worker_contents(udf_kind).await?;

    trace!("building {udf_kind} '{udf_name}'");

    let package_root_path = project.grafbase_directory_path.as_path();
    let udf_input_file_path_without_extension = match udf_kind {
        UdfKind::Resolver => project.resolvers_source_path.join(udf_name),
        UdfKind::Authorizer => project.authorizers_source_path.join(udf_name),
    };

    let udf_build_artifact_directory_path = match udf_kind {
        UdfKind::Resolver => project.resolvers_build_artifact_path.join(udf_name),
        UdfKind::Authorizer => project.authorizers_build_artifact_path.join(udf_name),
    };

    let mut udf_input_file_path = None;
    for extension in EXTENSIONS {
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

    trace!("locating package.json…");

    let package_json_path = {
        let paths = futures_util::stream::iter(
            udf_input_file_path
                .ancestors()
                .skip(1)
                .take_while(|path| path.starts_with(&project.path))
                .map(|directory_path| directory_path.join("package.json")),
        )
        .filter_map(|path| async {
            if tokio::fs::metadata(&path).await.is_ok() {
                Some(path)
            } else {
                None
            }
        });
        pin_mut!(paths);
        paths.next().await
    };

    let package_manager = (|| async {
        let package_json_path = package_json_path.as_deref()?;
        if tokio::fs::try_exists(&package_json_path).await.ok()? {
            let (guessed_from_package_json, guessed_from_package_root) = futures_util::join!(
                guess_package_manager_from_package_json(package_json_path),
                guess_package_manager_from_package_root(package_root_path)
            );
            guessed_from_package_json.or(guessed_from_package_root)
        } else {
            None
        }
    })()
    .await
    .unwrap_or(JavaScriptPackageManager::Npm);

    tokio::fs::create_dir_all(&udf_build_artifact_directory_path)
        .await
        .map_err(|_err| UdfBuildError::CreateDir(udf_build_artifact_directory_path.clone(), udf_kind))?;
    let udf_build_entrypoint_path = udf_build_artifact_directory_path.join("entrypoint.js");

    let udf_build_package_json_path = udf_build_artifact_directory_path.join("package.json");

    let artifact_directory_modules_path = udf_build_artifact_directory_path.join("node_modules");
    let artifact_directory_modules_path_string = artifact_directory_modules_path
        .to_str()
        .expect("must be valid if `artifact_directory_path_string` is valid");

    if let Some(package_json_file_path) = package_json_path.as_deref() {
        trace!("copying package.json from {}", package_json_file_path.display());
        tokio::fs::copy(package_json_file_path, &udf_build_package_json_path)
            .await
            .map_err(|err| UdfBuildError::CreateUdfArtifactFile(package_json_file_path.to_owned(), udf_kind, err))?;

        let artifact_directory_path_string = udf_build_artifact_directory_path
            .to_str()
            .ok_or_else(|| UdfBuildError::PathError(udf_build_artifact_directory_path.to_string_lossy().to_string()))?;

        let arguments = match package_manager {
            JavaScriptPackageManager::Npm => vec!["--prefix", artifact_directory_path_string, "install"],
            JavaScriptPackageManager::Pnpm => vec!["install"],
            JavaScriptPackageManager::Yarn => {
                vec!["install", "--modules-folder", artifact_directory_modules_path_string]
            }
        };
        run_command(
            package_manager,
            &arguments,
            &udf_build_artifact_directory_path,
            tracing,
            &[],
        )
        .await?;
    }

    // FIXME: This is probably rather fragile. Need to re-check why the wrangler build isn't propagating search paths properly.
    let udf_js_file_path = match udf_kind {
        UdfKind::Resolver => udf_build_artifact_directory_path
            .join("resolver")
            .with_extension(udf_input_file_path.extension().unwrap()),
        UdfKind::Authorizer => udf_build_artifact_directory_path
            .join("auth")
            .with_extension(udf_input_file_path.extension().unwrap()),
    };

    trace!("Copying the main file of the {udf_kind}");

    tokio::fs::copy(&udf_input_file_path, &udf_js_file_path)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_input_file_path, udf_kind, err))?;

    let udf_wrapper_worker_contents = udf_wrapper_worker_contents.replace(
        "${UDF_MAIN_FILE_PATH}",
        udf_js_file_path.to_slash().expect("must be valid UTF-8").as_ref(),
    );
    tokio::fs::write(&udf_build_entrypoint_path, udf_wrapper_worker_contents)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_build_entrypoint_path.clone(), udf_kind, err))?;

    let wrangler_output_directory_path = udf_build_artifact_directory_path.join("wrangler");
    let outdir_argument = format!(
        "--outdir={}",
        wrangler_output_directory_path.to_str().expect("must be valid utf-8"),
    );

    trace!("writing the package.json file for '{udf_name}' used by wrangler");

    let mut package_json = if package_json_path.is_some() {
        let package_json_contents = tokio::fs::read(&udf_build_package_json_path)
            .await
            .map_err(|err| UdfBuildError::ReadFile(udf_build_package_json_path.clone(), err))?;
        serde_json::from_slice(&package_json_contents).expect("must be valid JSON")
    } else {
        serde_json::json!({})
    };
    package_json.as_object_mut().expect("must be an object").insert(
        "module".to_owned(),
        serde_json::Value::String("wrangler/entrypoint.js".to_owned()),
    );
    package_json
        .as_object_mut()
        .expect("must be an object")
        .insert("type".to_owned(), serde_json::Value::String("module".to_owned()));

    let new_package_json_contents = serde_json::to_string_pretty(&package_json).expect("must be valid JSON");
    trace!("new package.json contents:\n{new_package_json_contents}");

    tokio::fs::write(&udf_build_package_json_path, new_package_json_contents)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_build_package_json_path.clone(), udf_kind, err))?;

    let wrangler_toml_file_path = udf_build_artifact_directory_path.join("wrangler.toml");

    let _: Result<_, _> = tokio::fs::remove_file(&wrangler_toml_file_path).await;

    // Not great. We use wrangler to produce the JS file that is then used as the input for the udf-specific worker.
    // FIXME: Swap out for the internal logic that wrangler effectively uses under the hood.
    {
        let wrangler_arguments = &[
            "exec",
            "--",
            "wrangler",
            "publish",
            "--dry-run",
            &outdir_argument,
            "--compatibility-date",
            "2023-05-14",
            "--name",
            "STUB",
            udf_build_entrypoint_path.to_str().expect("must be valid utf-8"),
        ];
        let wrangler_environment = &[
            ("CLOUDFLARE_API_TOKEN", "STUB"),
            ("FORCE_COLOR", "0"),
            ("NODE_PATH", artifact_directory_modules_path_string),
            ("WRANGLER_LOG", if tracing { "warn" } else { "error" }),
            ("WRANGLER_SEND_METRICS", "false"),
        ];
        run_command(
            JavaScriptPackageManager::Npm,
            wrangler_arguments,
            &environment.wrangler_installation_path,
            tracing,
            wrangler_environment,
        )
        .await
        .map_err(|err| match err {
            JavascriptPackageManagerComamndError::OutputError(_, output) => {
                UdfBuildError::UdfBuild(udf_kind, udf_name.to_owned(), output)
            }
            other => other.into(),
        })?;
    }

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
                &tokio::fs::read(wrangler_output_directory_path.join("entrypoint.js"))
                    .await
                    .expect("must succeed"),
            )
            .await
            .map_err(|err| UdfBuildError::CreateNotWriteToTemporaryFile(temp_file_path.to_path_buf(), err))?;
    }
    let entrypoint_js_path = wrangler_output_directory_path.join("entrypoint.js");
    tokio::fs::copy(temp_file_path, &entrypoint_js_path)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(entrypoint_js_path.clone(), udf_kind, err))?;

    let slugified_udf_name = slug::slugify(udf_name);
    tokio::fs::write(
        &wrangler_toml_file_path,
        format!(
            r#"
                name = "{slugified_udf_name}"
                [build.upload]
                format = "modules"
                [miniflare]
                routes = ["127.0.0.1/invoke"]
            "#,
        ),
    )
    .await
    .map_err(UdfBuildError::CreateTemporaryFile)?;

    Ok((udf_build_package_json_path, wrangler_toml_file_path))
}

pub async fn install_wrangler(environment: &Environment, tracing: bool) -> Result<(), ServerError> {
    let lock_file_path = environment.user_dot_grafbase_path.join(".wrangler.install.lock");
    let mut lock_file = tokio::task::spawn_blocking(move || {
        let mut file = fslock::LockFile::open(&lock_file_path)?;
        file.lock()?;
        Ok(file)
    })
    .await?
    .map_err(ServerError::Lock)?;

    info!("Installing wrangler…");
    tokio::fs::create_dir_all(&environment.wrangler_installation_path)
        .await
        .map_err(|_| ServerError::CreateDir(environment.wrangler_installation_path.clone()))?;
    // Install wrangler once and for all.
    run_command(
        JavaScriptPackageManager::Npm,
        &["add", "--save-dev", "wrangler@2"],
        environment.wrangler_installation_path.to_str().expect("must be valid"),
        tracing,
        &[],
    )
    .await?;
    run_command(
        JavaScriptPackageManager::Npm,
        &["install"],
        environment.wrangler_installation_path.to_str().expect("must be valid"),
        tracing,
        &[],
    )
    .await?;

    tokio::task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(ServerError::Unlock)?;

    Ok(())
}
