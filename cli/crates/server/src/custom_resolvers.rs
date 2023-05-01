use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc::Sender;

use common::environment::Environment;
use futures_util::pin_mut;
use itertools::Itertools;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::errors::ServerError;
use crate::servers::DetectedResolver;
use crate::types::ServerMessage;

async fn run_command<P: AsRef<Path>>(
    command_type: JavaScriptPackageManager,
    arguments: &[&str],
    current_directory: P,
    tracing: bool,
    environment: &[(&'static str, &str)],
) -> Result<(), ServerError> {
    trace!("running '{command_type} {}'", arguments.iter().format(" "));

    let command = Command::new(command_type.to_string())
        .envs(environment.iter().copied())
        .args(arguments)
        .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .current_dir(current_directory.as_ref())
        .spawn()
        .map_err(|err| ServerError::ResolverPackageManagerCommandError(command_type, err))?;

    let output = command
        .wait_with_output()
        .await
        .map_err(|err| ServerError::ResolverPackageManagerCommandError(command_type, err))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(ServerError::ResolverPackageManagerError(
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

#[allow(clippy::too_many_lines)]
async fn build_resolver(
    environment: &Environment,
    environment_variables: &std::collections::HashMap<String, String>,
    resolver_name: &str,
    resolver_wrapper_worker_contents: &str,
    resolver_build_artifact_directory_path: &Path,
    tracing: bool,
) -> Result<(), ServerError> {
    const EXTENSIONS: [&str; 2] = ["js", "ts"];

    use futures_util::StreamExt;

    trace!("building resolver {resolver_name}");

    let package_root_path = environment.project_grafbase_path.as_path();
    let resolver_input_file_path_without_extension = environment.resolvers_source_path.join(resolver_name);

    let resolver_paths = futures_util::stream::iter(
        EXTENSIONS
            .iter()
            .map(|extension| resolver_input_file_path_without_extension.with_extension(*extension)),
    )
    .filter(|path| {
        let path = path.as_path().to_owned();
        async move { tokio::fs::try_exists(path).await.expect("must succeed") }
    });
    futures_util::pin_mut!(resolver_paths);
    let resolver_input_file_path = resolver_paths
        .next()
        .await
        .ok_or_else(|| ServerError::ResolverDoesNotExist(resolver_input_file_path_without_extension.clone()))?;

    trace!("locating package.json…");

    let package_json_path = {
        let paths = futures_util::stream::iter(
            resolver_input_file_path
                .ancestors()
                .skip(1)
                .take_while(|path| path.starts_with(&environment.project_path))
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

    tokio::fs::create_dir_all(&resolver_build_artifact_directory_path)
        .await
        .map_err(ServerError::CreateTemporaryFile)?;
    let resolver_build_entrypoint_path = resolver_build_artifact_directory_path.join("entrypoint.js");

    if let Some(package_json_file_path) = package_json_path.as_deref() {
        trace!("copying package.json from {}", package_json_file_path.display());
        tokio::fs::copy(
            package_json_file_path,
            resolver_build_artifact_directory_path.join("package.json"),
        )
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;
    }

    let artifact_directory_path_string = resolver_build_artifact_directory_path
        .to_str()
        .ok_or(ServerError::CachePath)?;

    let artifact_directory_modules_path = resolver_build_artifact_directory_path.join("node_modules");
    let artifact_directory_modules_path_string =
        artifact_directory_modules_path.to_str().ok_or(ServerError::CachePath)?;

    {
        let arguments = match package_manager {
            JavaScriptPackageManager::Npm => vec![
                "--prefix",
                artifact_directory_path_string,
                "add",
                "--save-dev",
                "wrangler",
            ],
            JavaScriptPackageManager::Pnpm => {
                vec!["add", "-D", "wrangler"]
            }
            JavaScriptPackageManager::Yarn => {
                vec![
                    "add",
                    "--modules-folder",
                    artifact_directory_modules_path_string,
                    "-D",
                    "wrangler",
                ]
            }
        };
        run_command(
            package_manager,
            &arguments,
            resolver_build_artifact_directory_path,
            tracing,
            &[],
        )
        .await?;
    }

    {
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
            resolver_build_artifact_directory_path,
            tracing,
            &[],
        )
        .await?;
    }

    // FIXME: This is probably rather fragile. Need to re-check why the wrangler build isn't propagating search paths properly.
    let resolver_js_file_path = resolver_build_artifact_directory_path
        .join("resolver")
        .with_extension(resolver_input_file_path.extension().unwrap());

    trace!("Copying the main file of the resolver");

    tokio::fs::copy(resolver_input_file_path, &resolver_js_file_path)
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;

    let entrypoint_contents = resolver_wrapper_worker_contents.replace(
        "${RESOLVER_MAIN_FILE_PATH}",
        resolver_js_file_path.to_str().expect("must be valid utf-8"),
    );
    tokio::fs::write(&resolver_build_entrypoint_path, entrypoint_contents)
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;

    let wrangler_output_directory_path = resolver_build_artifact_directory_path.join("wrangler");
    let outdir_argument = format!(
        "--outdir={}",
        wrangler_output_directory_path.to_str().expect("must be valid utf-8"),
    );

    trace!("writing the package.json file for '{resolver_name}' used by wrangler");

    let package_json_contents = tokio::fs::read(resolver_build_artifact_directory_path.join("package.json"))
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;
    let mut package_json: serde_json::Value =
        serde_json::from_slice(&package_json_contents).expect("must be valid JSON");
    package_json.as_object_mut().expect("must be an object").insert(
        "module".to_owned(),
        serde_json::Value::String("wrangler/entrypoint.js".to_owned()),
    );

    let new_package_json_contents = serde_json::to_string_pretty(&package_json).expect("must be valid JSON");
    trace!("new package.json contents:\n{new_package_json_contents}");

    tokio::fs::write(
        resolver_build_artifact_directory_path.join("package.json"),
        new_package_json_contents,
    )
    .await
    .map_err(ServerError::CreateResolverArtifactFile)?;

    let wrangler_toml_file_path = resolver_build_artifact_directory_path.join("wrangler.toml");

    let _: Result<_, _> = tokio::fs::remove_file(&wrangler_toml_file_path).await;

    let wrangler_arguments = &[
        "wrangler",
        "publish",
        "--dry-run",
        &outdir_argument,
        "--compatibility-date",
        "2023-02-08",
        "--name",
        "STUB",
        resolver_build_entrypoint_path.to_str().expect("must be valid utf-8"),
    ];

    let wrangler_environment = &[
        ("CLOUDFLARE_API_TOKEN", "STUB"),
        ("FORCE_COLOR", "0"),
        ("WRANGLER_SEND_METRICS", "false"),
    ];

    // Not great. We use wrangler to produce the JS file that is then used as the input for the resolver-specific worker.
    // FIXME: Swap out for the internal logic that wrangler effectively uses under the hood.
    let mut arguments = match package_manager {
        JavaScriptPackageManager::Npm => vec!["--prefix", artifact_directory_path_string, "exec", "--"],
        JavaScriptPackageManager::Pnpm => vec!["exec"],
        JavaScriptPackageManager::Yarn => vec!["run"],
    };
    arguments.extend(wrangler_arguments);

    run_command(
        package_manager,
        &arguments,
        resolver_build_artifact_directory_path,
        tracing,
        wrangler_environment,
    )
    .await
    .map_err(|err| match err {
        ServerError::ResolverPackageManagerError(_, output) => {
            ServerError::ResolverBuild(resolver_name.to_owned(), output)
        }
        other => other,
    })?;

    let process_env_prelude = format!(
        "globalThis.process = {{ env: {} }};",
        serde_json::to_string(&environment_variables).expect("must be valid JSON")
    );

    let (temp_file, temp_file_path) = tokio::task::spawn_blocking(tempfile::NamedTempFile::new)
        .await?
        .map_err(ServerError::CreateResolverArtifactFile)?
        .into_parts();
    {
        let mut temp_file: tokio::fs::File = temp_file.into();
        temp_file
            .write_all(process_env_prelude.as_bytes())
            .await
            .map_err(ServerError::CreateResolverArtifactFile)?;
        temp_file
            .write_all(
                &tokio::fs::read(wrangler_output_directory_path.join("entrypoint.js"))
                    .await
                    .expect("must succeed"),
            )
            .await
            .map_err(ServerError::CreateResolverArtifactFile)?;
    }
    tokio::fs::copy(temp_file_path, wrangler_output_directory_path.join("entrypoint.js"))
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;

    let slugified_resolver_name = slug::slugify(resolver_name);
    tokio::fs::write(
        wrangler_toml_file_path,
        format!(
            r#"
                name = "{slugified_resolver_name}"
                [build.upload]
                format = "modules"
                [miniflare]
                routes = ["127.0.0.1/resolver/{resolver_name}/invoke"]
            "#,
        ),
    )
    .await
    .map_err(ServerError::CreateTemporaryFile)?;

    Ok(())
}

async fn extract_resolver_wrapper_worker_contents() -> Result<String, ServerError> {
    trace!("extracting resolver wrapper worker contents");
    let environment = Environment::get();
    tokio::fs::read_to_string(
        environment
            .user_dot_grafbase_path
            .join("custom-resolvers/wrapper-worker.js"),
    )
    .await
    .map_err(ServerError::SchemaParserResultRead)
}

pub async fn build_resolvers(
    sender: &Sender<ServerMessage>,
    environment: &Environment,
    environment_variables: &std::collections::HashMap<String, String>,
    resolvers: impl IntoIterator<Item = crate::servers::DetectedResolver>,
    tracing: bool,
) -> Result<HashMap<String, PathBuf>, ServerError> {
    use futures_util::{StreamExt, TryStreamExt};

    const RESOLVER_BUILD_CONCURRENCY: usize = 8;

    let mut resolvers_iterator = resolvers.into_iter().peekable();
    if resolvers_iterator.peek().is_none() {
        return Ok(HashMap::new());
    }

    let resolver_wrapper_worker_contents = extract_resolver_wrapper_worker_contents().await?;

    let resolvers_build_artifact_directory_path = environment.resolvers_build_artifact_path.as_path();

    futures_util::stream::iter(resolvers_iterator)
        .map(Ok)
        .map_ok(|DetectedResolver { resolver_name, fresh }| {
            let resolver_wrapper_worker_contents = resolver_wrapper_worker_contents.as_str();

            async move {
                let resolver_build_artifact_directory_path =
                    resolvers_build_artifact_directory_path.join(&resolver_name);
                if fresh {
                    let wrangler_toml_path = resolver_build_artifact_directory_path.join("wrangler.toml");
                    // Only touch the file to ensure the modified time relation is preserved.
                    tokio::task::spawn_blocking(|| {
                        filetime::set_file_mtime(wrangler_toml_path, filetime::FileTime::now()).expect("must succeed");
                    })
                    .await
                    .expect("must succeed");
                } else {
                    let start = std::time::Instant::now();
                    let _: Result<_, _> = sender.send(ServerMessage::StartResolverBuild(resolver_name.clone()));
                    build_resolver(
                        environment,
                        environment_variables,
                        resolver_name.as_str(),
                        resolver_wrapper_worker_contents,
                        &resolver_build_artifact_directory_path,
                        tracing,
                    )
                    .await?;
                    let _: Result<_, _> = sender.send(ServerMessage::CompleteResolverBuild {
                        name: resolver_name.clone(),
                        duration: start.elapsed(),
                    });
                }
                Ok((resolver_name, resolver_build_artifact_directory_path))
            }
        })
        .try_buffer_unordered(RESOLVER_BUILD_CONCURRENCY)
        .try_collect()
        .await
}
