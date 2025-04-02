mod server;

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use axum::Router;
use engine::Runtime;
use gateway_config::ModelControlProtocolConfig;
use rmcp::transport::{SseServer, sse_server::SseServerConfig};
use server::McpServer;
use tokio_util::sync::CancellationToken;

use super::gateway::EngineWatcher;

pub(super) fn router<S, R: Runtime>(
    engine: EngineWatcher<R>,
    config: &ModelControlProtocolConfig,
) -> (Router<S>, CancellationToken) {
    let (sse_server, router) = SseServer::new(SseServerConfig {
        // we never actually bind to a socket, it's just this weird API we need to obey
        bind: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)),
        sse_path: config.path.clone(),
        post_path: config.path.clone(),
        ct: CancellationToken::new(),
        sse_keep_alive: Some(Duration::from_secs(5)),
    });

    let instructions = config.instructions.clone();
    let auth = config.authentication.as_ref().map(|a| a.to_string());
    let enable_mutations = config.enable_mutations;

    let ct = sse_server
        .with_service(move || McpServer::new(engine.clone(), instructions.clone(), auth.clone(), enable_mutations));

    let router: Router<S> = router.with_state(());

    (router, ct)
}
