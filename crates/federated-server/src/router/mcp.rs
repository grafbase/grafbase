use std::{borrow::Cow, sync::Arc};

use axum::{Router, body::Bytes};
use engine::{Runtime, Schema};
use gateway_config::{ContractsConfig, MCPConfig};
use grafbase_mcp::GraphQLServer;
use http::request::Parts;
use tokio_util::sync::CancellationToken;

use crate::router::EngineWatcher;

pub(super) async fn router(
    engine: &EngineWatcher<impl Runtime>,
    contracts_config: &ContractsConfig,
    mcp_config: &MCPConfig,
) -> anyhow::Result<(Router, Option<CancellationToken>)> {
    grafbase_mcp::router(
        EngineGraphqlServer {
            watcher: engine.clone(),
            contracts_config: contracts_config.clone(),
        },
        mcp_config,
    )
    .await
}

struct EngineGraphqlServer<R: Runtime> {
    watcher: EngineWatcher<R>,
    contracts_config: ContractsConfig,
}

impl<R: Runtime> Clone for EngineGraphqlServer<R> {
    fn clone(&self) -> Self {
        Self {
            watcher: self.watcher.clone(),
            contracts_config: self.contracts_config.clone(),
        }
    }
}

impl<R: Runtime> GraphQLServer for EngineGraphqlServer<R> {
    async fn default_schema(&self) -> anyhow::Result<Arc<Schema>> {
        let engine = self.watcher.borrow().clone();
        Ok(match self.contracts_config.default_key {
            Some(ref key) => engine
                .get_engine_for_contract(key)
                .await
                .map_err(|err| {
                    anyhow::anyhow!(
                        err.errors
                            .into_iter()
                            .next()
                            .map(|err| err.message)
                            .unwrap_or_else(|| Cow::Borrowed("Unknown error"))
                    )
                })?
                .schema
                .clone(),
            None => engine.no_contract.schema.clone(),
        })
    }

    async fn get_schema_for_request(&self, parts: &Parts) -> anyhow::Result<Arc<Schema>> {
        let engine = self.watcher.borrow().clone();
        engine
            .get_schema(parts)
            .await
            .map_err(|err| anyhow::anyhow!(err.into_owned()))
    }

    async fn execute(&self, parts: Parts, body: Bytes) -> anyhow::Result<Bytes> {
        let engine = self.watcher.borrow().clone();
        let body_future = async move { Ok(body) };
        let request = http::Request::from_parts(parts, body_future);
        let response = engine.execute(request).await;
        let bytes = response
            .into_body()
            .into_bytes()
            .expect("Subscriptions are not supported through MCP.");
        Ok(bytes)
    }
}
