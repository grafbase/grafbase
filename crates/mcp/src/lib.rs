#![deny(unused_crate_dependencies)]
use engine_schema::Schema;
use grafbase_workspace_hack as _;
use http::request::Parts;
use quick_cache as _;
use tokio as _;
use tokio_stream as _;

pub mod server;
mod tools;

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use axum::{Router, body::Bytes};
use gateway_config::MCPConfig;
use rmcp::transport::{
    sse_server::{SseServer, SseServerConfig},
    streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager},
};
use tokio_util::sync::CancellationToken;

pub trait GraphQLServer: Send + Sync + 'static + Clone {
    fn default_schema(&self) -> impl Future<Output = anyhow::Result<Arc<Schema>>> + Send;
    fn get_schema_for_request(&self, parts: &Parts) -> impl Future<Output = anyhow::Result<Arc<Schema>>> + Send;
    fn execute(&self, parts: Parts, body: Bytes) -> impl Future<Output = anyhow::Result<Bytes>> + Send;
}

pub async fn router(
    gql: impl GraphQLServer,
    config: &MCPConfig,
) -> anyhow::Result<(Router, Option<CancellationToken>)> {
    let mcp_server = server::McpServer::new(gql, config.can_mutate).await?;
    match config.transport {
        gateway_config::McpTransport::StreamingHttp => {
            let service = StreamableHttpService::new(
                move || Ok(mcp_server.clone()),
                Arc::new(NeverSessionManager::default()),
                StreamableHttpServerConfig {
                    sse_keep_alive: Some(Duration::from_secs(5)),
                    stateful_mode: false,
                },
            );

            Ok((Router::new().route_service(&config.path, service), None))
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

            let ct = sse_server.with_service(move || mcp_server.clone());

            Ok((router, Some(ct)))
        }
    }
}
