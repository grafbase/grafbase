use std::sync::Mutex;

use async_trait::async_trait;
use dynaql::registry::{MetaField, MetaType, Registry};

use crate::{rules::visitor::VisitorContext, OpenApiDirective};

/// Provides the connector sub-parsers to the parsing process.
///
/// Allows us to inject the connector parsers into the parsing process,
/// or mock them out in tests as appropriate.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ConnectorParsers: Sync + Send {
    async fn fetch_and_parse_openapi(&self, openapi_directive: OpenApiDirective) -> Result<Registry, Vec<String>>;
}

/// A mock impl of the Connectors trait for tests and when we don't really care about
/// connector parsing
#[derive(Debug, Default)]
pub struct MockConnectorParsers {
    pub(crate) openapi_directives: Mutex<Vec<OpenApiDirective>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ConnectorParsers for MockConnectorParsers {
    async fn fetch_and_parse_openapi(&self, openapi_directive: OpenApiDirective) -> Result<Registry, Vec<String>> {
        self.openapi_directives.lock().unwrap().push(openapi_directive);

        Ok(Registry::new())
    }
}

/// Merges a bunch of registries into our vistor context
///
/// This allows each connector to get it's own registry so we can process them in paralell,
/// avoiding problems with multiple concurrent &mut Registry, or having to fuck about with
/// mutexes etc.
pub(crate) fn merge_registries(ctx: &mut VisitorContext<'_>, src_registries: Vec<Registry>) {
    for mut src_registry in src_registries {
        ctx.queries.extend(type_fields(
            src_registry.types.remove(&src_registry.query_type).unwrap(),
        ));

        if let Some(mutation_type) = &src_registry.mutation_type {
            ctx.mutations
                .extend(type_fields(src_registry.types.remove(mutation_type).unwrap()));
        }

        let mut main_registry = ctx.registry.borrow_mut();

        // I am sort of making the assumption that connectors won't generate names that
        // clash with each other here.  This should be the case with the type prefixes
        // in openapi but might need revisited with other connectors.
        main_registry.types.extend(src_registry.types.into_iter());
        main_registry.schemas.extend(src_registry.schemas.into_iter());

        main_registry.http_headers.extend(src_registry.http_headers.into_iter());

        // There are other fields on a Registry, but I think these are the only
        // ones likely to be touched by connectors for now.  We can look to update
        // this later as we add more connectors.
    }
}

#[allow(clippy::panic)]
fn type_fields(src_type: MetaType) -> impl Iterator<Item = MetaField> {
    let MetaType::Object { fields, .. } = src_type else {
        panic!("type_fields should only be called on objects")
    };
    fields.into_iter().map(|(_, field)| field)
}
