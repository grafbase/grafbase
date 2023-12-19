use std::{
    ffi, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use common::environment::{Project, SchemaLocation};
use tokio::task::JoinHandle;
use tokio_stream::{wrappers::errors::BroadcastStreamRecvError, StreamExt};

use crate::{
    config::ConfigStream,
    file_watcher::ChangeStream,
    types::{MessageSender, ServerMessage},
};

static CODEGEN_WORKER_HANDLE: OnceLock<JoinHandle<()>> = OnceLock::new();

/// Spawns the background worker responsible for generating TS type definitions for resolvers.
pub(crate) fn start_codegen_worker(
    file_changes: ChangeStream,
    config_changes: ConfigStream,
    message_sender: MessageSender,
) -> Result<(), ()> {
    let handle = tokio::spawn(async move { codegen_worker_task(file_changes, config_changes, message_sender).await });
    CODEGEN_WORKER_HANDLE.set(handle).map_err(|_| ())
}

enum CodegenIncomingEvent {
    ConfigChange,
    FileChange(Result<PathBuf, BroadcastStreamRecvError>),
}

async fn codegen_worker_task(file_changes: ChangeStream, config_changes: ConfigStream, message_sender: MessageSender) {
    let project = Project::get();
    let schema_path = &project.schema_path;
    let generated_ts_resolver_types_path = project.generated_directory_path().join("index.ts");
    let sdl_location = project.sdl_location();
    let resolvers_path = project.udfs_source_path(common_types::UdfKind::Resolver);

    let mut last_seen_sdl = None;

    // Try generating types on start up.
    if let SchemaLocation::Graphql(schema_path) = schema_path.location() {
        if let Some(sdl) = read_sdl(schema_path, &mut last_seen_sdl) {
            generate_ts_resolver_types(sdl, &generated_ts_resolver_types_path);
        }
    }

    let mut stream = config_changes
        .map(|_| CodegenIncomingEvent::ConfigChange)
        .merge(file_changes.into_stream().map(CodegenIncomingEvent::FileChange));

    while let Some(next) = stream.next().await {
        match next {
            CodegenIncomingEvent::ConfigChange => {
                if let Some(sdl) = read_sdl(&sdl_location, &mut last_seen_sdl) {
                    generate_ts_resolver_types(sdl, &generated_ts_resolver_types_path);
                };
            }
            CodegenIncomingEvent::FileChange(Ok(path)) => {
                if path.extension() == Some(ffi::OsStr::new("ts"))
                    && path.ancestors().any(|ancestor| ancestor == resolvers_path)
                {
                    // A resolver changed, check it.
                    let Some(sdl) = last_seen_sdl.as_ref() else {
                        continue;
                    };
                    if let Err(err) = typed_resolvers::check_resolver(sdl, &path) {
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

fn read_sdl<'a>(path: &Path, last_seen_sdl: &'a mut Option<String>) -> Option<&'a str> {
    let sdl = fs::read_to_string(path).ok()?;
    last_seen_sdl.replace(sdl);
    last_seen_sdl.as_deref()
}

fn generate_ts_resolver_types(graphql_sdl: &str, target_file_path: &Path) {
    // Here we write to a string first, because in case code generation fails, we don't want to
    // replace the existing types with an empty file.
    let mut out = String::new();
    if typed_resolvers::generate_ts_resolver_types(graphql_sdl, &mut out).is_ok() {
        fs::create_dir_all(target_file_path.parent().unwrap()).ok();
        fs::write(target_file_path, &out).ok();
    }
}
