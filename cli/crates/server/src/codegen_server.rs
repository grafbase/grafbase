use std::{
    ffi, fs, iter,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use common::environment::Project;
use engine::registry::{resolvers::Resolver, RegistrySdlExt};
use tokio::task::JoinHandle;
use tokio_stream::{wrappers::errors::BroadcastStreamRecvError, StreamExt};

use crate::{
    config::{Config, ConfigActor, ConfigStream},
    file_watcher::ChangeStream,
    types::{MessageSender, ServerMessage},
};

/// Spawns the background worker responsible for generating TS type definitions for resolvers.
pub(crate) fn start_codegen_worker(
    file_changes: ChangeStream,
    config_actor: &ConfigActor,
    message_sender: MessageSender,
) {
    static CODEGEN_WORKER_HANDLE: OnceLock<JoinHandle<()>> = OnceLock::new();

    let project = Project::get();

    let initial_config = config_actor
        .current_result()
        .as_ref()
        .map(|config| transform_config(config, project))
        .ok()
        .flatten();
    let config_changes = config_actor.config_stream();

    CODEGEN_WORKER_HANDLE.get_or_init(|| {
        tokio::spawn(async move {
            codegen_worker_task(file_changes, initial_config, config_changes, message_sender, project).await
        })
    });
}

enum CodegenIncomingEvent {
    ConfigChange(Config),
    FileChange(Result<PathBuf, BroadcastStreamRecvError>),
}

async fn codegen_worker_task(
    file_changes: ChangeStream,
    mut last_seen_config: Option<TransformedConfig>,
    config_changes: ConfigStream,
    message_sender: MessageSender,
    project: &Project,
) {
    let resolvers_path = project.udfs_source_path(common_types::UdfKind::Resolver);

    // Try generating types on start up.
    if let Some(TransformedConfig {
        sdl,
        generated_ts_resolver_types_path: resolver_codegen_path,
        resolvers,
    }) = last_seen_config.as_ref()
    {
        generate_ts_resolver_types(sdl, resolvers, resolver_codegen_path);
    }

    let mut stream = config_changes
        .map(CodegenIncomingEvent::ConfigChange)
        .merge(file_changes.into_stream().map(CodegenIncomingEvent::FileChange));

    while let Some(next) = stream.next().await {
        match next {
            CodegenIncomingEvent::ConfigChange(config) => {
                last_seen_config = transform_config(&config, project);
                if let Some(TransformedConfig {
                    sdl,
                    generated_ts_resolver_types_path,
                    resolvers,
                }) = &last_seen_config
                {
                    generate_ts_resolver_types(sdl, resolvers, generated_ts_resolver_types_path);
                }
            }
            CodegenIncomingEvent::FileChange(Ok(path)) => {
                if path.extension() == Some(ffi::OsStr::new("ts"))
                    && path.ancestors().any(|ancestor| ancestor == resolvers_path)
                {
                    // A resolver changed, check it.
                    let Some(TransformedConfig { sdl, resolvers, .. }) = last_seen_config.as_ref() else {
                        continue;
                    };

                    let Ok(parsed_sdl) = typed_resolvers::parse_schema::<&str>(sdl) else {
                        continue;
                    };

                    let mut analyzed = typed_resolvers::analyze_schema(&parsed_sdl);

                    for resolver in resolvers {
                        analyzed.push_custom_resolver(resolver);
                    }

                    if let Err(err) = typed_resolvers::check_resolver(&path, &analyzed) {
                        message_sender
                            .send(ServerMessage::CompilationError(format!("{err:?}")))
                            .ok();
                    }
                }
            }
            CodegenIncomingEvent::FileChange(Err(_)) => (),
        };
    }
}

#[derive(Debug)]
struct TransformedConfig {
    sdl: String,
    generated_ts_resolver_types_path: PathBuf,
    resolvers: Vec<typed_resolvers::CustomResolver>,
}

fn transform_config(config: &Config, project: &Project) -> Option<TransformedConfig> {
    let codegen_config = config.registry.codegen.as_ref()?;

    if !codegen_config.enabled {
        return None;
    }

    let federation = false; // not relevant for codegen
    let sdl = config.registry.export_sdl(federation);
    let schema_parent_dir = project.schema_path.parent().expect("the schema path to have a parent");

    Some(TransformedConfig {
        sdl,
        generated_ts_resolver_types_path: schema_parent_dir
            .join(codegen_config.path.as_deref().unwrap_or("generated"))
            .join("index.ts"),
        resolvers: find_resolvers(config),
    })
}

fn generate_ts_resolver_types(sdl: &str, resolvers: &[typed_resolvers::CustomResolver], target_file_path: &Path) {
    let Ok(parsed) = typed_resolvers::parse_schema(sdl) else {
        return;
    };

    let mut analyzed_schema = typed_resolvers::analyze_schema(&parsed);

    for resolver in resolvers {
        analyzed_schema.push_custom_resolver(resolver);
    }

    // Here we write to a string first, because in case code generation fails, we don't want to
    // replace the existing types with an empty file.
    let mut out = String::new();

    if typed_resolvers::generate_ts_resolver_types(&analyzed_schema, &mut out).is_ok() {
        fs::create_dir_all(target_file_path.parent().unwrap()).ok();
        fs::write(target_file_path, &out).ok();
    }
}

fn find_resolvers(config: &Config) -> Vec<typed_resolvers::CustomResolver> {
    config
        .registry
        .types
        .values()
        .filter_map(|ty| Some(ty.name()).zip(ty.fields()))
        .flat_map(|(name, fields)| iter::repeat(name).zip(fields.values()))
        .filter_map(|(parent_type_name, field)| match &field.resolver {
            Resolver::CustomResolver(resolver) => Some(typed_resolvers::CustomResolver {
                resolver_name: resolver.resolver_name.clone(),
                field_name: field.name.clone(),
                parent_type_name: parent_type_name.to_owned(),
            }),
            _ => None,
        })
        .collect()
}
