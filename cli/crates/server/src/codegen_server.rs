use std::{
    ffi, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use common::environment::{Project, SchemaLocation};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

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
    let handle = tokio::spawn(async move { codegen_worker_loop(file_changes, config_changes, message_sender).await });
    CODEGEN_WORKER_HANDLE.set(handle).map_err(|_| ())
}

async fn codegen_worker_loop(file_changes: ChangeStream, config_changes: ConfigStream, message_sender: MessageSender) {
    let project = Project::get();
    let schema_path = &project.schema_path;
    let generated_ts_resolver_types_path = project.generated_directory_path().join("index.ts");

    let mut last_seen_sdl = None;

    // Try generating types on start up.
    if let SchemaLocation::Graphql(schema_path) = schema_path.location() {
        if let Some(sdl) = read_sdl(schema_path, &mut last_seen_sdl) {
            generate_ts_resolver_types(sdl, &generated_ts_resolver_types_path);
        }
    }

    let _ = tokio::join!(
        tokio::spawn(type_generate_loop(config_changes, generated_ts_resolver_types_path)),
        tokio::spawn(resolver_check_loop(file_changes, message_sender, last_seen_sdl))
    );
}

async fn type_generate_loop(mut config_changes: ConfigStream, destination_path: PathBuf) {
    let sdl_location = Project::get().sdl_location();
    let sdl_location = sdl_location.to_str().expect("utf8 paths");

    while (config_changes.next().await).is_some() {
        generate_ts_resolver_types(sdl_location, &destination_path)
    }
}

async fn resolver_check_loop(
    mut file_changes: ChangeStream,
    message_sender: MessageSender,
    last_seen_sdl: Option<String>,
) {
    let project = Project::get();
    let resolvers_path = project.udfs_source_path(common_types::UdfKind::Resolver);

    while let Some(path) = file_changes.next().await {
        if path.extension() == Some(ffi::OsStr::new("ts"))
            && path.ancestors().any(|ancestor| ancestor == resolvers_path)
        {
            // A resolver changed, check it.
            let Some(sdl) = last_seen_sdl.as_deref() else {
                continue;
            };
            if let Err(err) = typed_resolvers::check_resolver(sdl, &path) {
                message_sender
                    .send(ServerMessage::CompilationError(format!("{err:?}")))
                    .ok();
            }
        }
    }
}

fn read_sdl<'a>(path: &Path, last_seen_sdl: &'a mut Option<String>) -> Option<&'a str> {
    let sdl = fs::read_to_string(path).ok()?;
    *last_seen_sdl = Some(sdl);
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
