use std::{borrow::Cow, sync::Mutex};

use async_trait::async_trait;
use engine::{
    registry::{self, MetaField, MetaType, Registry},
    Name, Pos, Positioned,
};
use engine_parser::types::TypeDefinition;

use crate::{
    rules::{postgres_directive::PostgresDirective, visitor::VisitorContext},
    GraphqlDirective, OpenApiDirective,
};

/// Provides the connector sub-parsers to the parsing process.
///
/// Allows us to inject the connector parsers into the parsing process,
/// or mock them out in tests as appropriate.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ConnectorParsers: Sync + Send {
    async fn fetch_and_parse_openapi(&self, directive: OpenApiDirective) -> Result<Registry, Vec<String>>;
    async fn fetch_and_parse_graphql(&self, directive: GraphqlDirective) -> Result<Registry, Vec<String>>;
    async fn fetch_and_parse_postgres(&self, directive: &PostgresDirective) -> Result<Registry, Vec<String>>;
}

/// A mock impl of the Connectors trait for tests and when we don't really care about
/// connector parsing
#[derive(Debug, Default)]
pub struct MockConnectorParsers {
    pub(crate) openapi_directives: Mutex<Vec<OpenApiDirective>>,
    pub(crate) graphql_directives: Mutex<Vec<GraphqlDirective>>,
    pub(crate) postgres_directives: Mutex<Vec<PostgresDirective>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ConnectorParsers for MockConnectorParsers {
    async fn fetch_and_parse_openapi(&self, directive: OpenApiDirective) -> Result<Registry, Vec<String>> {
        self.openapi_directives.lock().unwrap().push(directive);

        Ok(Registry::new())
    }

    async fn fetch_and_parse_graphql(&self, directive: GraphqlDirective) -> Result<Registry, Vec<String>> {
        self.graphql_directives.lock().unwrap().push(directive);

        Ok(Registry::new())
    }

    async fn fetch_and_parse_postgres(&self, directive: &PostgresDirective) -> Result<Registry, Vec<String>> {
        self.postgres_directives.lock().unwrap().push(directive.clone());

        Ok(Registry::new())
    }
}

/// Merges a bunch of registries into our vistor context
///
/// This allows each connector to get it's own registry so we can process them in paralell,
/// avoiding problems with multiple concurrent &mut Registry, or having to deal with
/// mutexes etc.
pub(crate) fn merge_registry(ctx: &mut VisitorContext<'_>, mut src_registry: Registry, position: Pos) {
    ctx.queries.extend(type_fields(
        src_registry.types.remove(&src_registry.query_type).unwrap(),
    ));

    if let Some(mutation_type) = &src_registry.mutation_type {
        ctx.mutations
            .extend(type_fields(src_registry.types.remove(mutation_type).unwrap()));
    }

    // The parser relies on `ctx.types` in a few places, which contains parsed SDL
    // TypeDefinitions (rather than the processed MetaTypes that our connectors give
    // us).  We hackishly fake these TypeDefinitions here to work around that
    ctx.types.extend(
        src_registry
            .types
            .iter()
            .map(|(name, ty)| (name.clone(), Cow::Owned(meta_type_to_type_definition(ty, position)))),
    );

    let mut main_registry = ctx.registry.borrow_mut();

    // I am sort of making the assumption that connectors won't generate names that
    // clash with each other here.  This should be the case with the type prefixes
    // in openapi but might need revisited with other connectors.
    main_registry.types.extend(src_registry.types.into_iter());

    main_registry.implements.extend(src_registry.implements.into_iter());
    main_registry.http_headers.extend(src_registry.http_headers.into_iter());
    main_registry.postgres_databases.extend(src_registry.postgres_databases);

    // There are other fields on a Registry, but I think these are the only
    // ones likely to be touched by connectors for now.  We can look to update
    // this later as we add more connectors.
}

#[allow(clippy::panic)]
fn type_fields(src_type: MetaType) -> impl Iterator<Item = MetaField> {
    let MetaType::Object(registry::ObjectType { fields, .. }) = src_type else {
        panic!("type_fields should only be called on objects")
    };
    fields.into_iter().map(|(_, field)| field)
}

fn meta_type_to_type_definition(ty: &MetaType, position: Pos) -> Positioned<TypeDefinition> {
    use engine_parser::types::{EnumType, InputObjectType, InterfaceType, ObjectType, TypeKind, UnionType};

    Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(Name::new(ty.name()), position),
            directives: vec![],
            // These are just meant to be dummy entries to keep the parser happy
            // that they types exist so I"m not filling most of the details in for now
            kind: match ty {
                MetaType::Scalar { .. } => TypeKind::Scalar,
                MetaType::Object { .. } => TypeKind::Object(ObjectType {
                    implements: vec![],
                    fields: vec![],
                }),
                MetaType::Interface { .. } => TypeKind::Interface(InterfaceType {
                    implements: vec![],
                    fields: vec![],
                }),
                MetaType::Union { .. } => TypeKind::Union(UnionType { members: vec![] }),
                MetaType::Enum { .. } => TypeKind::Enum(EnumType { values: vec![] }),
                MetaType::InputObject { .. } => TypeKind::InputObject(InputObjectType { fields: vec![] }),
            },
        },
        position,
    )
}
