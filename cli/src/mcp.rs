use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::body::Bytes;
use crossterm::{
    QueueableCommand,
    cursor::MoveUp,
    style::Stylize,
    terminal::{Clear, ClearType},
};
use engine::Schema;
use http::{HeaderMap, request::Parts};
use std::io::stdout;
use url::Url;

use crate::{cli_input::McpCommand, dev::DEFAULT_PORT};

#[tokio::main(flavor = "multi_thread")]
pub(crate) async fn run(args: McpCommand) -> anyhow::Result<()> {
    let schema = if let Some(path) = &args.schema {
        std::fs::read_to_string(path).map_err(|err| anyhow::anyhow!("Could not read {}: {err}", path.display()))?
    } else {
        println!("{} your endpoint...\n", "Introspecting".yellow().bold());
        let schema = grafbase_graphql_introspection::introspect(args.url.as_str(), &args.headers().collect::<Vec<_>>())
            .await
            .map_err(|err| anyhow::anyhow!("Introspection: {err}"))?;
        stdout().queue(MoveUp(2))?.queue(Clear(ClearType::CurrentLine))?;
        tracing::debug!("Introspected GraphQL\n:{schema}");
        schema
    };

    println!("{} the MCP server...\n", "Preparing".yellow().bold());
    let schema = engine::Schema::builder(&schema)
        .for_operation_analytics_only()
        .build()
        .await
        .map_err(|err| anyhow::anyhow!("Internal: failed to build schema: {err}"))?;

    let mcp_config = gateway_config::MCPConfig {
        enabled: true,
        can_mutate: args.execute_mutations,
        transport: match args.transport {
            crate::cli_input::McpTransport::StreamingHttp => gateway_config::McpTransport::StreamingHttp,
            crate::cli_input::McpTransport::Sse => gateway_config::McpTransport::Sse,
        },
        ..Default::default()
    };

    let mut headers = HeaderMap::new();
    for (name, value) in args.headers() {
        headers.insert(
            name.parse::<http::HeaderName>()
                .map_err(|err| anyhow::anyhow!("Invalid header name '{name}': {err}"))?,
            value
                .parse()
                .map_err(|err| anyhow::anyhow!("Invalid header value '{value}': {err}"))?,
        );
    }

    let (router, ct) = grafbase_mcp::router(GraphqlProxy::new(schema, args.url, headers), &mcp_config).await?;

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, args.port.unwrap_or(DEFAULT_PORT)));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Shutting down...");
        if let Some(ct) = ct {
            ct.cancel();
        }
    });

    stdout().queue(MoveUp(2))?.queue(Clear(ClearType::CurrentLine))?;
    println!("MCP server exposed at http://{}{}", addr, mcp_config.path);

    if let Err(e) = server.await {
        tracing::error!(error = %e, "Shutdown with error");
    }

    Ok(())
}

#[derive(Clone)]
struct GraphqlProxy(Arc<GraphQLProxyInner>);

struct GraphQLProxyInner {
    schema: Arc<Schema>,
    url: Url,
    headers: HeaderMap,
    client: reqwest::Client,
}

impl std::ops::Deref for GraphqlProxy {
    type Target = GraphQLProxyInner;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl GraphqlProxy {
    fn new(schema: Schema, url: Url, mut headers: HeaderMap) -> Self {
        let client = runtime_local::fetch::client_builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        headers.insert(http::header::CONNECTION, http::HeaderValue::from_static("keep-alive"));

        Self(Arc::new(GraphQLProxyInner {
            schema: Arc::new(schema),
            url,
            headers,
            client,
        }))
    }
}

impl grafbase_mcp::GraphQLServer for GraphqlProxy {
    async fn default_schema(&self) -> anyhow::Result<Arc<Schema>> {
        Ok(self.schema.clone())
    }

    async fn get_schema_for_request(&self, _parts: &Parts) -> anyhow::Result<Arc<Schema>> {
        Ok(self.schema.clone())
    }

    async fn execute(&self, parts: Parts, body: Bytes) -> anyhow::Result<Bytes> {
        let response = self
            .client
            .post(self.url.clone())
            .headers(self.headers.clone())
            .headers(parts.headers)
            .body(body)
            .send()
            .await?;
        let bytes = response.bytes().await?;
        Ok(bytes)
    }
}
