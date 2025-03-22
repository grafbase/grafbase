mod coerce;
mod context;
mod error;
mod extension;
mod graph;
mod interner;
mod subgraphs;

use context::Context;
use extension_catalog::ExtensionCatalog;
use subgraphs::SubgraphsContext;

use self::error::*;
pub(crate) use coerce::*;
pub(crate) use graph::*;

pub use self::error::BuildError;

use crate::*;

pub(crate) async fn build(
    config: &gateway_config::Config,
    federated_graph: &federated_graph::FederatedGraph,
    extension_catalog: &ExtensionCatalog,
    version: Version,
) -> Result<Schema, BuildError> {
    Context::new(config, extension_catalog, federated_graph)
        .await?
        .build(version)
}

impl Context<'_> {
    fn build(mut self, version: Version) -> Result<Schema, BuildError> {
        let default_headers = &self.config.headers;
        let default_header_rules = self.ingest_header_rules(default_headers);
        let (
            Context {
                strings,
                urls,
                config,
                subgraphs:
                    SubgraphsContext {
                        graphql_endpoints,
                        virtual_subgraphs,
                        ..
                    },
                regexps,
                header_rules,
                templates,
                ..
            },
            graph,
            introspection,
        ) = self.into_ctx_graph_introspection()?;

        let subgraphs = SubGraphs {
            graphql_endpoints,
            virtual_subgraphs,
            introspection,
        };

        let response_extension = config
            .telemetry
            .exporters
            .response_extension
            .clone()
            .unwrap_or_default()
            .into();

        let executable_document_limit_bytes = config
            .executable_document_limit
            .bytes()
            .try_into()
            .expect("executable document limit should not be negative");

        let settings = PartialConfig {
            timeout: config.gateway.timeout,
            default_header_rules,
            operation_limits: config.operation_limits.unwrap_or_default(),
            disable_introspection: !config.graph.introspection.unwrap_or_default(),
            retry: config.gateway.retry.enabled.then_some(config.gateway.retry.into()),
            batching: config.gateway.batching.clone(),
            complexity_control: (&config.complexity_control).into(),
            response_extension,
            apq_enabled: config.apq.enabled,
            executable_document_limit_bytes,
            trusted_documents: config.trusted_documents.clone().into(),
            websocket_forward_connection_init_payload: config.websockets.forward_connection_init_payload,
        };

        let strings = strings
            .into_iter()
            .map(|mut s| {
                s.shrink_to_fit();
                s
            })
            .collect();

        Ok(Schema {
            subgraphs,
            graph,
            version,
            strings,
            regexps: regexps.into(),
            urls: urls.into(),
            templates,
            header_rules,
            settings,
        })
    }
}
