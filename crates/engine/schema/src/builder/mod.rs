mod coerce;
mod context;
mod error;
mod extension;
mod graph;
mod hash;
mod interner;
pub(crate) mod mutable;
mod sdl;
mod subgraphs;

use std::borrow::Cow;

use context::{BuildContext, Interners};
use extension::ExtensionsContext;
use extension_catalog::ExtensionCatalog;
use gateway_config::Config;
use sdl::{Sdl, SpanTranslator};

pub(crate) use coerce::*;
pub(crate) use error::*;
pub(crate) use graph::*;

use crate::*;

pub struct Builder<'a> {
    pub sdl: &'a str,
    pub config: Option<&'a gateway_config::Config>,
    pub extension_catalog: Option<&'a ExtensionCatalog>,
    pub for_operation_analytics_only: bool,
}

impl<'a> Builder<'a> {
    pub fn new(sdl: &'a str) -> Self {
        Self {
            sdl,
            config: None,
            extension_catalog: None,
            for_operation_analytics_only: false,
        }
    }

    pub fn config<'b, 'out>(self, config: &'b gateway_config::Config) -> Builder<'out>
    where
        'a: 'out,
        'b: 'out,
    {
        Builder {
            config: Some(config),
            ..self
        }
    }

    pub fn extensions<'b, 'out>(self, catalog: &'b ExtensionCatalog) -> Builder<'out>
    where
        'a: 'out,
        'b: 'out,
    {
        Builder {
            extension_catalog: Some(catalog),
            ..self
        }
    }

    pub fn for_operation_analytics_only(self) -> Self {
        Builder {
            for_operation_analytics_only: true,
            ..self
        }
    }

    pub async fn build(self) -> Result<Schema, String> {
        let sdl = self.sdl;
        self.build_inner().await.map_err(|err| {
            let translator = SpanTranslator::new(sdl);
            err.into_string(&translator)
        })
    }

    async fn build_inner(self) -> Result<Schema, Error> {
        let Self {
            sdl,
            config,
            extension_catalog,
            for_operation_analytics_only,
        } = self;
        let config = config.map(Cow::Borrowed).unwrap_or_default();
        let extension_catalog = extension_catalog.map(Cow::Borrowed).unwrap_or_default();

        if !sdl.trim().is_empty() {
            let doc = &cynic_parser::parse_type_system_document(sdl).map_err(|err| Error::from(err.to_string()))?;
            let sdl = Sdl::try_from((sdl, doc))?;
            let extensions = if for_operation_analytics_only {
                ExtensionsContext::empty_with_catalog(&extension_catalog)
            } else {
                ExtensionsContext::load(&sdl, &extension_catalog).await?
            };

            BuildContext::new(&sdl, &extensions, &config)?.build(for_operation_analytics_only)
        } else {
            let sdl = Default::default();
            let extensions = if for_operation_analytics_only {
                ExtensionsContext::empty_with_catalog(&extension_catalog)
            } else {
                ExtensionsContext::load(&sdl, &extension_catalog).await?
            };

            BuildContext::new(&sdl, &extensions, &config)?.build(for_operation_analytics_only)
        }
    }
}

impl BuildContext<'_> {
    fn build(self, for_operation_analytics_only: bool) -> Result<Schema, Error> {
        let (mut graph_builder, introspection) = ingest_definitions(self)?;

        // From this point on the definitions should have been all added and now we interpret the
        // directives.
        ingest_directives(&mut graph_builder, for_operation_analytics_only)?;

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
            selections,
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

        let mut schema = Schema {
            subgraphs,
            graph,
            hash,
            selections: selections.inner,
            extensions,
            strings,
            regexps: regexps.into(),
            urls: urls.into(),
            config: settings,
        };
        mutable::mark_builtins_and_introspection_as_accessible(&mut schema);

        Ok(schema)
    }
}

fn build_settings(config: &Config) -> PartialConfig {
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
        contract_cache_max_size: config.graph.contracts.cache.max_size,
    }
}
