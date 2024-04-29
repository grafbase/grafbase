//! Data structues for resolvers.
//!
//! Actual logic should not be implemented in this crate, as many crates pull this in.
//! Implement your logic elsewhere (probably the engine crate or a crate the engine pulls in)
//!

pub mod atlas_data_api;
pub mod custom;
pub mod graphql;
pub mod http;
pub mod introspection;
pub mod join;
pub mod postgres;
pub mod transformer;
pub mod variable_resolve_definition;

#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Resolver {
    // By default a resolver will just return its parent value
    #[default]
    Parent,
    Typename,
    Transformer(transformer::Transformer),
    CustomResolver(custom::CustomResolver),
    Composition(Vec<Resolver>),
    Http(Box<http::HttpResolver>),
    Graphql(Box<graphql::Resolver>),
    MongoResolver(atlas_data_api::AtlasDataApiResolver),
    PostgresResolver(postgres::PostgresResolver),
    FederationEntitiesResolver,
    Introspection(introspection::IntrospectionResolver),
    Join(join::JoinResolver),
}

impl Resolver {
    pub fn and_then(mut self, resolver: impl Into<Resolver>) -> Self {
        let resolver = resolver.into();
        match &mut self {
            Resolver::Composition(resolvers) => {
                resolvers.push(resolver);
                self
            }
            _ => Resolver::Composition(vec![self, resolver]),
        }
    }

    pub fn and_then_maybe(self, resolver: Option<impl Into<Resolver>>) -> Self {
        match resolver {
            Some(other) => self.and_then(other),
            None => self,
        }
    }

    pub fn is_parent(&self) -> bool {
        matches!(self, Self::Parent)
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::CustomResolver(_))
    }

    pub fn is_join(&self) -> bool {
        matches!(self, Self::Join(_))
    }
}
