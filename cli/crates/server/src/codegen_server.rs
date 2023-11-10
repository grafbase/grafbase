use crate::{event::Event, types::ServerMessage};
use common::environment::{Project, SchemaLocation};
use std::{ffi, fs, path::Path, sync::OnceLock};
use tokio::{
    sync::{broadcast, mpsc::UnboundedSender},
    task::JoinHandle,
};

static CODEGEN_WORKER_HANDLE: OnceLock<JoinHandle<()>> = OnceLock::new();

/// Spawns the background worker responsible for generating TS type definitions for resolvers.
pub(crate) fn start_codegen_worker(
    server_events: broadcast::Receiver<Event>,
    event_sender: UnboundedSender<ServerMessage>,
) -> Result<(), ()> {
    let handle = tokio::spawn(async move { codegen_worker_loop(server_events, event_sender).await });
    CODEGEN_WORKER_HANDLE.set(handle).map_err(|_| ())
}

async fn codegen_worker_loop(
    mut server_events: broadcast::Receiver<Event>,
    event_sender: UnboundedSender<ServerMessage>,
) {
    let project = Project::get();
    let resolvers_path = project.udfs_source_path(common_types::UdfKind::Resolver);
    let schema_path = &project.schema_path;
    let mut last_seen_sdl = None;
    let generated_ts_resolver_types_path = project.path.join("generated/index.ts");

    // Try generating types on start up.
    if let SchemaLocation::Graphql(schema_path) = schema_path.location() {
        if let Some(sdl) = read_sdl(schema_path, &mut last_seen_sdl) {
            generate_ts_resolver_types(sdl, &generated_ts_resolver_types_path);
        }
    }

    loop {
        match server_events.recv().await {
            Err(_err) => {
                break; // channel is broken, we might as well stop trying
            }
            Ok(Event::Reload(path)) => {
                let path = path.canonicalize().unwrap_or(path);
                match schema_path.location() {
                    SchemaLocation::Graphql(schema_path) if path == schema_path.as_path() => {
                        // The SDL schema was edited. Regenerate types.

                        let Some(sdl) = read_sdl(&path, &mut last_seen_sdl) else {
                            continue;
                        };

                        generate_ts_resolver_types(sdl, &generated_ts_resolver_types_path);
                    }
                    _ if path.extension() == Some(ffi::OsStr::new("ts"))
                        && path.ancestors().any(|ancestor| ancestor == resolvers_path) =>
                    {
                        // A resolver changed, check it.
                        let Some(sdl) = last_seen_sdl.as_deref() else {
                            continue;
                        };
                        if let Err(err) = typed_resolvers::check_resolver(sdl, &path) {
                            event_sender
                                .send(ServerMessage::CompilationError(format!("{err:?}")))
                                .ok();
                        }
                    }
                    _ => (),
                };
            }
            Ok(Event::NewSdlFromTsConfig(path)) => {
                let Some(sdl) = read_sdl(&path, &mut last_seen_sdl) else {
                    continue;
                };
                generate_ts_resolver_types(sdl, &generated_ts_resolver_types_path);
            }
            Ok(_) => {}
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
