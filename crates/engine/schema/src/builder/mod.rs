mod coerce;
mod context;
mod error;
mod extension;
mod graph;
mod hash;
mod interner;
mod sdl;
mod subgraphs;

use context::{BuildContext, Interners};
use extension::{ExtensionsContext, finalize_selection_set_resolvers, ingest_extension_schema_directives};
use extension_catalog::ExtensionCatalog;
use sdl::{Sdl, SpanTranslator};

pub(crate) use coerce::*;
pub(crate) use error::*;
pub(crate) use graph::*;

use crate::*;

pub(crate) async fn build(
    config: &gateway_config::Config,
    sdl: &str,
    extension_catalog: &ExtensionCatalog,
) -> Result<Schema, String> {
    build_inner(config, sdl, extension_catalog).await.map_err(|err| {
        let translator = SpanTranslator::new(sdl);
        err.into_string(&translator)
    })
}

async fn build_inner(
    config: &gateway_config::Config,
    sdl: &str,
    extension_catalog: &ExtensionCatalog,
) -> Result<Schema, Error> {
    if !sdl.trim().is_empty() {
        let doc = &cynic_parser::parse_type_system_document(sdl).map_err(|err| Error::from(err.to_string()))?;
        let sdl = Sdl::try_from((sdl, doc))?;
        let extensions = ExtensionsContext::load(&sdl, extension_catalog).await?;

        BuildContext::new(&sdl, &extensions, config)?.build()
    } else {
        let sdl = Default::default();
        let extensions = ExtensionsContext::load(&sdl, extension_catalog).await?;

        BuildContext::new(&sdl, &extensions, config)?.build()
    }
}

impl BuildContext<'_> {
    fn build(self) -> Result<Schema, Error> {
        let (mut graph_builder, sdl_definitions, introspection) = ingest_definitions(self)?;

        // From this point on the definitions should have been all added and now we interpret the
        // directives.

        ingest_extension_schema_directives(&mut graph_builder)?;

        ingest_directives(&mut graph_builder, &sdl_definitions)?;

        finalize_selection_set_resolvers(&mut graph_builder)?;

        let GraphBuilder {
            ctx:
                BuildContext {
                    sdl,
                    interners,
                    subgraphs,
                    config,
                    extensions: ExtensionsContext { catalog, .. },
                    ..
                },
            mut graph,
            required_scopes,
            ..
        } = graph_builder;
        graph.required_scopes = required_scopes.into();

        let subgraphs = subgraphs.finalize_with(introspection);
        let settings = build_settings(config);

        let Interners { strings, regexps, urls } = interners;

        let strings = strings
            .into_iter()
            .map(|mut s| {
                s.shrink_to_fit();
                s
            })
            .collect();

        let extensions = catalog.iter().map(|ext| ext.manifest.id.clone()).collect();
        let hash = hash::compute(sdl, catalog);

        Ok(Schema {
            subgraphs,
            graph,
            hash,
            extensions,
            strings,
            regexps: regexps.into(),
            urls: urls.into(),
            settings,
        })
    }
}

fn build_settings(config: &gateway_config::Config) -> PartialConfig {
    PartialConfig {
        timeout: config.gateway.timeout,
        operation_limits: config.operation_limits.unwrap_or_default(),
        disable_introspection: !config.graph.introspection.unwrap_or_default(),
        retry: config.gateway.retry.enabled.then_some(config.gateway.retry.into()),
        batching: config.gateway.batching.clone(),
        complexity_control: (&config.complexity_control).into(),
        response_extension: config
            .telemetry
            .exporters
            .response_extension
            .clone()
            .unwrap_or_default()
            .into(),
        apq_enabled: config.apq.enabled,
        executable_document_limit_bytes: config
            .executable_document_limit
            .bytes()
            .try_into()
            .expect("executable document limit should not be negative"),
        trusted_documents: config.trusted_documents.clone().into(),
        websocket_forward_connection_init_payload: config.websockets.forward_connection_init_payload,
    }
}
