use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use common::environment::Environment;
use futures_util::{pin_mut, TryStreamExt};
use tokio::process::Command;

use crate::errors::ServerError;

#[derive(strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum CommandType {
    Npm,
    Npx,
}

async fn run_npm_command<P: AsRef<Path>>(
    command_type: CommandType,
    artifact_directory_path: P,
    extra_arguments: &[&str],
    tracing: bool,
    environment: &[(&'static str, &'static str)],
) -> Result<(), ServerError> {
    use itertools::Itertools;

    let artifact_directory_path_string = artifact_directory_path
        .as_ref()
        .to_str()
        .ok_or(ServerError::CachePath)?;

    let mut arguments = vec!["--prefix", artifact_directory_path_string];
    arguments.extend(extra_arguments);

    trace!("running '{command_type} {}'", arguments.iter().format(" "));

    let npm_command = Command::new(command_type.as_ref())
        .envs(environment.iter().copied())
        .args(arguments)
        .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .current_dir(artifact_directory_path)
        .spawn()
        .map_err(ServerError::NpmCommandError)?;

    let output = npm_command
        .wait_with_output()
        .await
        .map_err(ServerError::NpmCommandError)?;

    if output.status.success() {
        Ok(())
    } else {
        Err(ServerError::NpmCommand(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

async fn build_resolver(
    environment: &Environment,
    resolver_name: &str,
    resolver_wrapper_worker_contents: &str,
    tracing: bool,
) -> Result<PathBuf, ServerError> {
    use futures_util::StreamExt;

    trace!("building resolver {resolver_name}");

    let resolver_input_file_path = environment
        .resolvers_source_path
        .join(resolver_name)
        .with_extension("js");
    if tokio::fs::metadata(&resolver_input_file_path).await.is_err() {
        return Err(ServerError::ResolverDoesNotExist(resolver_input_file_path));
    }

    let package_json_file_path = {
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

    let resolvers_build_artifact_directory_path = environment.resolvers_build_artifact_path.as_path();
    let resolver_build_artifact_directory_path = resolvers_build_artifact_directory_path.join(resolver_name);
    tokio::fs::create_dir_all(&resolver_build_artifact_directory_path)
        .await
        .map_err(ServerError::CreateTemporaryFile)?;
    let resolver_build_entrypoint_path = resolver_build_artifact_directory_path.join("entrypoint.js");

    if let Some(package_json_file_path) = package_json_file_path {
        trace!("copying package.json from {}", package_json_file_path.display());
        tokio::fs::copy(
            package_json_file_path,
            resolver_build_artifact_directory_path.join("package.json"),
        )
        .await
        .map_err(ServerError::NpmCommandError)?;
    }

    run_npm_command(
        CommandType::Npm,
        resolvers_build_artifact_directory_path,
        &["add", "--save-dev", "wrangler"],
        tracing,
        &[],
    )
    .await?;
    run_npm_command(
        CommandType::Npm,
        resolvers_build_artifact_directory_path,
        &["install"],
        tracing,
        &[],
    )
    .await?;

    let entrypoint_contents = resolver_wrapper_worker_contents.replace(
        "${RESOLVER_MAIN_FILE_PATH}",
        resolver_input_file_path.to_str().expect("must be valid utf-8"),
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

    tokio::fs::write(
        resolver_build_artifact_directory_path.join("package.json"),
        r#"{ "module": "wrangler/entrypoint.js" }"#,
    )
    .await
    .map_err(ServerError::CreateResolverArtifactFile)?;

    // Not great. We use wrangler to produce the JS file that is then used as the input for the resolver-specific worker.
    // FIXME: Swap out for the internal logic that wrangler effectively uses under the hood.
    run_npm_command(
        CommandType::Npx,
        resolvers_build_artifact_directory_path,
        &[
            "wrangler",
            "publish",
            "--dry-run",
            &outdir_argument,
            "--compatibility-date",
            "2023-02-08",
            "--name",
            "STUB",
            resolver_build_entrypoint_path.to_str().expect("must be valid utf-8"),
        ],
        tracing,
        &[("FORCE_COLOR", "0"), ("CLOUDFLARE_API_TOKEN", "STUB")],
    )
    .await?;

    Ok(resolver_build_artifact_directory_path)
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
    environment: &Environment,
    resolvers: impl IntoIterator<Item = String>,
    tracing: bool,
) -> Result<HashMap<String, PathBuf>, ServerError> {
    use futures_util::StreamExt;

    let mut resolvers_iterator = resolvers.into_iter().peekable();
    if resolvers_iterator.peek().is_none() {
        return Ok(HashMap::new());
    }

    let resolver_wrapper_worker_contents = extract_resolver_wrapper_worker_contents().await?;

    futures_util::stream::iter(resolvers_iterator)
        .map(Ok)
        .and_then(|resolver_name| async {
            let output_file_path = build_resolver(
                environment,
                resolver_name.as_str(),
                &resolver_wrapper_worker_contents,
                tracing,
            )
            .await?;
            Ok((resolver_name, output_file_path))
        })
        .try_collect()
        .await
}
