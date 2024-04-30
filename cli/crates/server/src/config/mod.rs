use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::Ordering,
    time::{Duration, SystemTime},
};

use common::{
    consts::GRAFBASE_SCHEMA_FILE_NAME,
    environment::{Environment, Project, SchemaLocation},
};
use common_types::UdfKind;
use engine::Registry;
use futures_util::stream::BoxStream;
use itertools::Itertools as _;
use tokio::process::Command;

use crate::{
    atomics::REGISTRY_PARSED_EPOCH_OFFSET_MILLIS,
    bun::install_bun,
    consts::{CONFIG_PARSER_SCRIPT_CJS, CONFIG_PARSER_SCRIPT_ESM, ENTRYPOINT_SCRIPT_FILE_NAME, SCHEMA_PARSER_DIR},
    servers::EnvironmentName,
};

mod actor;
mod error;
mod parser;

pub use self::{actor::ConfigActor, error::ConfigError, parser::parse_sdl};

#[derive(Debug, Clone)]
pub struct DetectedUdf {
    pub udf_name: String,
    pub udf_kind: UdfKind,
    pub fresh: bool,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) registry: Registry,
    pub(crate) detected_udfs: Vec<DetectedUdf>,
    pub(crate) federated_graph_config: Option<parser_sdl::federation::FederatedGraphConfig>,

    // The file that triggered this change (if any)
    pub(crate) triggering_file: Option<PathBuf>,
}

pub type ConfigStream = BoxStream<'static, Config>;

/// Builds the configuration for the current project.
///
/// Either by building & running grafbase.config.ts or parsing grafbase.schema
pub(crate) async fn build_config(
    environment_variables: &HashMap<String, String>,
    triggering_file: Option<PathBuf>,
    environment_name: EnvironmentName,
) -> Result<Config, ConfigError> {
    trace!("parsing schema");
    let project = Project::get();

    let schema_path = match project.schema_path.location() {
        SchemaLocation::TsConfig(ref ts_config_path) => {
            install_bun().await?;
            let written_schema_path = parse_and_generate_config_from_ts(ts_config_path, environment_name).await?;

            Cow::Owned(written_schema_path)
        }
        SchemaLocation::Graphql(ref path) => Cow::Borrowed(path.to_str().ok_or(ConfigError::ProjectPath)?),
    };
    let schema = tokio::fs::read_to_string(Path::new(schema_path.as_ref())).await?;

    let parser::ParserResult {
        registry,
        required_udfs,
        federated_graph_config,
    } = parser::parse_sdl(&schema, environment_variables).await?;

    // Federated graphs have empty SDL schemas, from the config's perspective.
    if federated_graph_config.is_none() {
        validate_registry_sdl(&registry.export_sdl(registry.enable_federation))?;
    }

    let offset = REGISTRY_PARSED_EPOCH_OFFSET_MILLIS.load(Ordering::Acquire);
    let registry_mtime = SystemTime::UNIX_EPOCH.checked_add(Duration::from_millis(offset));
    let detected_resolvers = futures_util::future::join_all(required_udfs.into_iter().map(|(udf_kind, udf_name)| {
        // Last file to be written to in the build process.
        let entrypoint_path = project
            .udfs_build_artifact_path(udf_kind)
            .join(&udf_name)
            .join(ENTRYPOINT_SCRIPT_FILE_NAME);
        async move {
            let entrypoint_mtime = tokio::fs::metadata(&entrypoint_path)
                .await
                .ok()
                .map(|metadata| metadata.modified().expect("must be supported"));
            let fresh = registry_mtime
                .zip(entrypoint_mtime)
                .map(|(registry_mtime, entrypoint_mtime)| entrypoint_mtime > registry_mtime)
                .unwrap_or_default();
            DetectedUdf {
                udf_name,
                udf_kind,
                fresh,
            }
        }
    }))
    .await;

    if !detected_resolvers.is_empty() {
        // will immediately return if already installed for session
        install_bun().await?;
    }

    REGISTRY_PARSED_EPOCH_OFFSET_MILLIS.store(
        u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap(),
        Ordering::Release,
    );

    Ok(Config {
        registry,
        detected_udfs: detected_resolvers,
        federated_graph_config,
        triggering_file,
    })
}

fn validate_registry_sdl(schema: &str) -> Result<(), ConfigError> {
    let diagnostics = graphql_schema_validation::validate(schema);

    if diagnostics.has_errors() {
        Err(ConfigError::ParseSchema(
            diagnostics.iter().map(ToString::to_string).join("\n"),
        ))
    } else {
        Ok(())
    }
}

/// Parses a TypeScript Grafbase configuration and generates a GraphQL schema
/// file to the filesystem, returning a path to the generated file.
async fn parse_and_generate_config_from_ts(
    ts_config_path: &Path,
    environment_name: EnvironmentName,
) -> Result<String, ConfigError> {
    let environment = Environment::get();
    let project = Project::get();

    let generated_config_path = project.dot_grafbase_directory_path.join(GRAFBASE_SCHEMA_FILE_NAME);

    if !project.dot_grafbase_directory_path.exists() {
        std::fs::create_dir_all(&project.dot_grafbase_directory_path)?;
    }

    let module_type = project
        .package_json_path
        .as_deref()
        .and_then(ModuleType::from_package_json)
        .unwrap_or_default();

    let config_parser_path = environment
        .user_dot_grafbase_path
        .join(SCHEMA_PARSER_DIR)
        .join(match module_type {
            ModuleType::CommonJS => CONFIG_PARSER_SCRIPT_CJS,
            ModuleType::Esm => CONFIG_PARSER_SCRIPT_ESM,
        });

    let args = &[
        "run".to_owned(),
        config_parser_path.to_string_lossy().to_string(),
        ts_config_path.to_string_lossy().to_string(),
        generated_config_path.to_string_lossy().to_string(),
    ];

    let bun_command = Command::new(
        environment
            .bun_executable_path()
            .map_err(|err| ConfigError::LoadTsConfig(err.to_string()))?,
    )
    .args(args)
    .env(
        "GRAFBASE_ENV",
        match environment_name {
            EnvironmentName::Production => "production",
            EnvironmentName::Dev => "dev",
            EnvironmentName::None => "",
        },
    )
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

    let output = bun_command.wait_with_output().await?;

    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        return Err(ConfigError::LoadTsConfig(msg.into_owned()));
    }

    let generated_config_path = generated_config_path.to_str().ok_or(ConfigError::ProjectPath)?;

    trace!("Generated configuration in {}.", generated_config_path);

    Ok(generated_config_path.to_string())
}

#[derive(Default)]
enum ModuleType {
    #[default]
    CommonJS,
    Esm,
}

impl ModuleType {
    pub fn from_package_json(package_json: &Path) -> Option<ModuleType> {
        let value = serde_json::from_slice::<serde_json::Value>(&std::fs::read(package_json).ok()?).ok()?;
        if value["type"].as_str()? == "module" {
            Some(ModuleType::Esm)
        } else {
            Some(ModuleType::CommonJS)
        }
    }
}
