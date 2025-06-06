#![deny(unused_crate_dependencies)]
use grafbase_workspace_hack as _;

mod server;
mod tools;

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use axum::Router;
use engine::{Engine, Runtime};
use gateway_config::ModelControlProtocolConfig;
use rmcp::transport::{
    sse_server::{SseServer, SseServerConfig},
    streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager},
};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

type EngineWatcher<R> = watch::Receiver<Arc<Engine<R>>>;

pub fn router<R: Runtime>(
    engine: EngineWatcher<R>,
    config: &ModelControlProtocolConfig,
) -> (Router, Option<CancellationToken>) {
    match config.transport {
        gateway_config::McpTransport::StreamingHttp => {
            let execute_mutations = config.execute_mutations;

            let mcp_server = server::McpServer::new(engine.clone(), execute_mutations).unwrap();

            let service = StreamableHttpService::new(
                move || mcp_server.clone(),
                Arc::new(LocalSessionManager::default()),
                StreamableHttpServerConfig {
                    sse_keep_alive: Some(Duration::from_secs(5)),
                    stateful_mode: true,
                },
            );

            (Router::new().route_service(&config.path, service), None)
        }
        gateway_config::McpTransport::Sse => {
            let (sse_server, router) = SseServer::new(SseServerConfig {
                // we never actually bind to a socket, it's just this weird API we need to obey
                bind: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)),
                sse_path: config.path.clone(),
                post_path: config.path.clone(),
                ct: CancellationToken::new(),
                sse_keep_alive: Some(Duration::from_secs(5)),
            });

            let execute_mutations = config.execute_mutations;

            let mcp_server = server::McpServer::new(engine.clone(), execute_mutations).unwrap();
            let ct = sse_server.with_service(move || mcp_server.clone());

            (router, Some(ct))
        }
    }
}
