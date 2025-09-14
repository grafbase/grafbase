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

use std::{borrow::Cow, sync::Arc};

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
    pub config: Option<Arc<Config>>,
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

    pub fn config(self, config: Arc<Config>) -> Builder<'a> {
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
        self.build_inner().await.map_err(|mut errors| {
            use std::fmt::Write;
            let translator = SpanTranslator::new(sdl);
            errors.sort_by_key(|err| err.span.map_or(0, |span| span.start));
            let mut out = String::with_capacity(errors.len() * 100);
            for err in errors {
                writeln!(&mut out, "{}", err.display(&translator)).unwrap();
            }
            out
        })
    }

    async fn build_inner(self) -> Result<Schema, Vec<Error>> {
        let Self {
            sdl,
            config,
            extension_catalog,
            for_operation_analytics_only,
        } = self;
        let config = config.unwrap_or_else(|| Arc::new(Config::default()));
        let extension_catalog = extension_catalog.map(Cow::Borrowed).unwrap_or_default();

        if !sdl.trim().is_empty() {
            let doc =
                &cynic_parser::parse_type_system_document(sdl).map_err(|err| vec![Error::from(err.to_string())])?;
            let sdl = Sdl::try_from((sdl, doc))?;
            let extensions = if for_operation_analytics_only {
                ExtensionsContext::empty_with_catalog(&extension_catalog)
            } else {
                ExtensionsContext::load(&sdl, &extension_catalog).await?
            };

            BuildContext::new(&sdl, &extensions, config).build(for_operation_analytics_only)
        } else {
            let sdl = Default::default();
            let extensions = if for_operation_analytics_only {
                ExtensionsContext::empty_with_catalog(&extension_catalog)
            } else {
                ExtensionsContext::load(&sdl, &extension_catalog).await?
            };

            BuildContext::new(&sdl, &extensions, config).build(for_operation_analytics_only)
        }
    }
}

impl BuildContext<'_> {
    fn build(self, for_operation_analytics_only: bool) -> Result<Schema, Vec<Error>> {
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
            graph,
            selections,
            ..
        } = graph_builder;

        let subgraphs = subgraphs.finalize_with(introspection);

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
            config: config.into(),
        };
        mutable::mark_builtins_and_introspection_as_accessible(&mut schema);

        Ok(schema)
    }
}
