use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveUp,
    style::Stylize,
    terminal::{Clear, ClearType},
};
use engine::{CachedOperation, RequestExtensions};
use gateway_config::{Config, HeaderForward, HeaderInsert, HeaderRule, NameOrPattern};
use grafbase_telemetry::metrics::{EngineMetrics, meter_from_global_provider};
use regex::Regex;
use runtime::{
    authentication::LegacyToken, entity_cache::EntityCache, rate_limiting::RateLimiter, trusted_documents_client,
};
use runtime_local::{InMemoryEntityCache, InMemoryOperationCache, NativeFetcher};
use std::io::stdout;
use wasi_component_loader::{WasmContext, extension::EngineWasmExtensions};

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
    let mut config = Config::default();
    config.headers.push(HeaderRule::Forward(HeaderForward {
        name: NameOrPattern::Pattern(Regex::new(r".*").unwrap().into()),
        default: None,
        rename: None,
    }));
    for (name, value) in args.headers() {
        config.headers.push(HeaderRule::Insert(HeaderInsert {
            name: name
                .parse()
                .map_err(|err| anyhow::anyhow!("Invalid header name '{name}': {err}"))?,
            value: value
                .parse()
                .map_err(|err| anyhow::anyhow!("Invalid header value '{value}': {err}"))?,
        }));
    }

    let extensions_catalog = Default::default();
    let schema = {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        subgraphs
            .ingest_str(&schema, "api", Some(args.url.as_str()))
            .map_err(|err| anyhow::anyhow!("Invalid schema: {err}"))?;
        let federated_graph = graphql_composition::compose(subgraphs)
            .into_result()
            .map_err(|diagnostics| {
                anyhow::anyhow!(
                    "Internal: failed to compose schemas: {}",
                    diagnostics.iter_messages().collect::<Vec<_>>().join("\n")
                )
            })?;
        let sdl = graphql_composition::render_federated_sdl(&federated_graph).expect("render_federated_sdl()");
        engine::Schema::builder(&sdl)
            .config(&config)
            .extensions(&extensions_catalog)
            .build()
            .await
            .map_err(|err| anyhow::anyhow!("Internal: failed to build schema: {err}"))?
    };

    let extensions = EngineWasmExtensions::default();
    let runtime = MinimalRuntime {
        fetcher: NativeFetcher::new(&config).unwrap(),
        trusted_documents: trusted_documents_client::Client::new(()),
        metrics: EngineMetrics::build(&meter_from_global_provider(), None),
        extensions,
        rate_limiter: Default::default(),
        entity_cache: Default::default(),
        operation_cache: Default::default(),
    };

    let engine = engine::ContractAwareEngine::new(Arc::new(schema), runtime);
    let (_, rx) = tokio::sync::watch::channel(Arc::new(engine));

    let mcp_config = gateway_config::ModelControlProtocolConfig {
        enabled: true,
        execute_mutations: args.execute_mutations,
        transport: match args.transport {
            crate::cli_input::McpTransport::StreamingHttp => gateway_config::McpTransport::StreamingHttp,
            crate::cli_input::McpTransport::Sse => gateway_config::McpTransport::Sse,
        },
        ..Default::default()
    };

    let (router, ct) = grafbase_mcp::router(&rx, &mcp_config);

    let router = router.layer(
        tower::ServiceBuilder::new().map_request(|mut request: axum::http::Request<_>| {
            // FIXME: Imitating the federated-server extension layer... not great
            request.extensions_mut().insert(RequestExtensions::<WasmContext> {
                context: Default::default(),
                token: LegacyToken::Anonymous,
                contract_key: None,
            });
            request
        }),
    );
    // Do something with the router, e.g., add routes or middleware

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

struct MinimalRuntime {
    fetcher: NativeFetcher,
    trusted_documents: trusted_documents_client::Client,
    metrics: EngineMetrics,
    extensions: EngineWasmExtensions,
    rate_limiter: RateLimiter,
    entity_cache: InMemoryEntityCache,
    operation_cache: InMemoryOperationCache<Arc<CachedOperation>>,
}

impl engine::Runtime for MinimalRuntime {
    type Fetcher = NativeFetcher;
    type OperationCache = InMemoryOperationCache<Arc<CachedOperation>>;
    type Extensions = EngineWasmExtensions;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn operation_cache(&self) -> &Self::OperationCache {
        &self.operation_cache
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    async fn sleep(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await
    }

    fn entity_cache(&self) -> &dyn EntityCache {
        &self.entity_cache
    }

    fn metrics(&self) -> &EngineMetrics {
        &self.metrics
    }

    fn extensions(&self) -> &Self::Extensions {
        &self.extensions
    }

    async fn clone_and_adjust_for_contract(&self, schema: &Arc<engine::Schema>) -> Result<Self, String> {
        Ok(MinimalRuntime {
            fetcher: self.fetcher.clone(),
            trusted_documents: self.trusted_documents.clone(),
            metrics: self.metrics.clone(),
            extensions: self
                .extensions
                .clone_and_adjust_for_contract(schema)
                .await
                .map_err(|err| err.to_string())?,
            rate_limiter: self.rate_limiter.clone(),
            entity_cache: InMemoryEntityCache::default(),
            operation_cache: InMemoryOperationCache::default(),
        })
    }
}
