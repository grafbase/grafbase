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
use engine::CachedOperation;
use engine_auth::AuthenticationService;
use gateway_config::{Config, HeaderInsert, HeaderRule};
use grafbase_telemetry::metrics::{EngineMetrics, meter_from_global_provider};
use runtime::{entity_cache::EntityCache, kv::KvStore, rate_limiting::RateLimiter, trusted_documents_client};
use runtime_local::{
    InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCache, NativeFetcher, wasi::hooks::HooksWasi,
};
use std::io::stdout;
use wasi_component_loader::extension::WasmExtensions;

use crate::{backend::dev::DEFAULT_PORT, cli_input::McpCommand};

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
        schema
    };

    println!("{} the MCP server...\n", "Preparing".yellow().bold());
    let mut config = Config::default();
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
        let federated_graph = graphql_composition::compose(&subgraphs)
            .into_result()
            .map_err(|diagnostics| {
                anyhow::anyhow!(
                    "Internal: failed to compose schemas: {}",
                    diagnostics.iter_messages().collect::<Vec<_>>().join("\n")
                )
            })?;
        let sdl = federated_graph::render_federated_sdl(&federated_graph).expect("render_federated_sdl()");
        let current_dir = std::env::current_dir().ok();
        engine::Schema::build(current_dir.as_deref(), &sdl, &config, &extensions_catalog)
            .await
            .map_err(|err| anyhow::anyhow!("Internal: failed to build schema: {err}"))?
    };

    let extensions = WasmExtensions::default();
    let kv = InMemoryKvStore::runtime();
    let authentication = AuthenticationService::new(&config, &extensions_catalog, extensions.clone(), &kv);
    let runtime = MinimalRuntime {
        fetcher: NativeFetcher::new(&config).unwrap(),
        trusted_documents: trusted_documents_client::Client::new(()),
        kv,
        metrics: EngineMetrics::build(&meter_from_global_provider(), None),
        hooks: Default::default(),
        extensions,
        rate_limiter: Default::default(),
        entity_cache: Default::default(),
        operation_cache: Default::default(),
        authentication,
    };

    let engine = engine::Engine::new(Arc::new(schema), runtime).await;
    let (_, rx) = tokio::sync::watch::channel(Arc::new(engine));

    let mcp_config = gateway_config::ModelControlProtocolConfig {
        enabled: true,
        execute_mutations: args.execute_mutations,
        ..Default::default()
    };

    let (router, ct) = grafbase_mcp::router(rx, &mcp_config);

    // Do something with the router, e.g., add routes or middleware

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, args.port.unwrap_or(DEFAULT_PORT)));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Shutting down...");
        ct.cancel();
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
    kv: KvStore,
    metrics: EngineMetrics,
    hooks: HooksWasi,
    extensions: WasmExtensions,
    rate_limiter: RateLimiter,
    entity_cache: InMemoryEntityCache,
    operation_cache: InMemoryOperationCache<Arc<CachedOperation>>,
    authentication: AuthenticationService<WasmExtensions>,
}

impl engine::Runtime for MinimalRuntime {
    type Hooks = HooksWasi;
    type Fetcher = NativeFetcher;
    type OperationCache = InMemoryOperationCache<Arc<CachedOperation>>;
    type Extensions = WasmExtensions;
    type Authenticate = AuthenticationService<Self::Extensions>;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn trusted_documents(&self) -> &trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn hooks(&self) -> &Self::Hooks {
        &self.hooks
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

    fn authentication(&self) -> &Self::Authenticate {
        &self.authentication
    }
}
