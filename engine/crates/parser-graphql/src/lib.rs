//! GraphQL schema parser for upstream APIs connected to Grafbase.
//!
//! The parser fetches a GraphQL schema from an upstream server, parses the response, and modifies
//! it to allow the result to be exposed through the Grafbase API.

use grafbase_workspace_hack as _;

mod conversion;

use cynic::{
    http::{CynicReqwestError, ReqwestExt},
    GraphQlError, QueryBuilder,
};
use cynic_introspection::{query::IntrospectionQuery, SchemaError};
use engine::registry::{resolvers::Resolver, Deprecation, MetaField, ObjectType, Registry};
use http::header::USER_AGENT;
use inflector::Inflector;
use registry_v2::{
    resolvers::{graphql, transformer::Transformer},
    ConnectorHeaders,
};
use url::Url;

static BUILTIN_DIRECTIVES: &[&str] = &["deprecated", "include", "skip", "specifiedBy"];
static BUILTIN_SCALARS: &[&str] = &["Boolean", "Float", "ID", "Int", "String"];

/// Parse the schema of an upstream GraphQL server, and return a pre-populated [`Registry`] with
/// the upstream schema details embedded.
///
/// The upstream server is exposed by adding a `namespace` field at the top-level Grafbase schema,
/// which exposes an object of fields representing the root-level fields of the upstream API.
///
/// As an example, given the namespace `bookstore` and this schema:
///
/// ```text
/// type Query {
///     books: [Book]
///     authors: [Author]
/// }
///
/// type Book {
///     title: String
///     author: Author
/// }
///
/// type Author {
///     name: String
///     books: [Book]
/// }
/// ```
///
/// The Grafbase API schema would become:
///
/// ```text
/// type Query {
///     bookstore: BookstoreQuery,
/// }
///
/// type BookstoreQuery {
///     books: [BookstoreBook]
///     authors: [BookstoreAuthor]
/// }
///
/// type BookstoreBook {
///     title: String
///     author: BookstoreAuthor
/// }
///
/// type BookstoreAuthor {
///     name: String
///     books: [BookstoreBook]
/// }
/// ```
///
/// Any provided `headers` are passed to the upstream API as HTTP request headers. These can be
/// used for authentication, etc.
///
/// # Errors
///
/// See [`Error`] for more details.
pub async fn parse_schema(
    client: reqwest::Client,
    name: &str,
    namespace: bool,
    url: &Url,
    headers: ConnectorHeaders,
    introspection_headers: impl IntoIterator<Item = (&str, &str)>,
    type_prefix: Option<&str>,
) -> Result<Registry, Vec<Error>> {
    let mut builder = client.post(url.clone()).header(USER_AGENT, "Grafbase");

    for (key, value) in introspection_headers {
        builder = builder.header(key, value);
    }

    let result = builder
        .run_graphql(IntrospectionQuery::build(()))
        .await
        .map_err(|err| vec![err.into()])?;

    if let Some(errors) = result.errors {
        return Err(errors.into_iter().map(Into::into).collect());
    }

    let Some(data) = result.data else {
        return Err(vec![Error::MissingData]);
    };

    let schema = data.into_schema().map_err(|err| vec![err.into()])?;

    let parser = Parser {
        name: name.to_string(),
        namespace,
        url: url.clone(),
        type_prefix: type_prefix
            .map(ToOwned::to_owned)
            .or(namespace.then(|| name.to_pascal_case())),
    };

    let mut registry = parser.into_registry(schema);
    registry.http_headers.insert(format!("GraphQLConnector{name}"), headers);

    Ok(registry)
}

struct Parser {
    name: String,
    namespace: bool,
    url: Url,
    type_prefix: Option<String>,
}

impl Parser {
    fn into_registry(self, mut schema: cynic_introspection::Schema) -> Registry {
        use cynic_introspection::Type;

        Self::filter_builtins(&mut schema);

        for ty in &mut schema.types {
            match ty {
                Type::Object(ref mut v) => self.update_object(v),
                Type::InputObject(v) => self.update_input_object(v),
                Type::Enum(v) => self.update_enum(v),
                Type::Interface(v) => self.update_interface(v),
                Type::Union(v) => self.update_union(v),
                Type::Scalar(v) => self.update_scalar(v),
            }
        }

        let mut registry = conversion::registry_from_introspection(schema);

        if self.namespace {
            self.add_root_query_field(&mut registry, &self.name);

            if registry.mutation_type.is_some() {
                self.add_root_mutation_field(&mut registry, &self.name);
            }
        } else {
            self.update_root_query_fields(&mut registry);

            if registry.mutation_type.is_some() {
                self.update_root_mutation_fields(&mut registry);
            }
        };

        Self::add_field_resolvers(&mut registry);

        registry
    }

    /// Rename object type from (e.g.) `MyObject` to `UpstreamMyObject`.
    ///
    /// Then, iterate all fields within the object, and perform any needed actions.
    fn update_object(&self, v: &mut cynic_introspection::ObjectType) {
        self.prefixed(&mut v.name);
        v.fields.iter_mut().for_each(|v| self.update_field(v));
        v.interfaces.iter_mut().for_each(|interface| self.prefixed(interface));
    }

    /// Similar to [`Parser::update_object()`], but for `InputObjectType`.
    fn update_input_object(&self, v: &mut cynic_introspection::InputObjectType) {
        self.prefixed(&mut v.name);
        v.fields.iter_mut().for_each(|v| self.update_input_value(v));
    }

    fn update_enum(&self, v: &mut cynic_introspection::EnumType) {
        self.prefixed(&mut v.name);
    }

    fn update_interface(&self, v: &mut cynic_introspection::InterfaceType) {
        self.prefixed(&mut v.name);
        v.fields.iter_mut().for_each(|v| self.update_field(v));
        v.possible_types
            .iter_mut()
            .for_each(|possible_type| self.prefixed(possible_type));
    }

    fn update_union(&self, v: &mut cynic_introspection::UnionType) {
        self.prefixed(&mut v.name);
        v.possible_types.iter_mut().for_each(|v| self.update_union_member(v));
    }

    fn update_union_member(&self, v: &mut String) {
        self.prefixed(v);
    }

    fn update_scalar(&self, v: &mut cynic_introspection::ScalarType) {
        self.prefixed(&mut v.name);
    }

    fn update_field(&self, v: &mut cynic_introspection::Field) {
        self.update_field_type(&mut v.ty);
        v.args.iter_mut().for_each(|v| self.update_input_value(v));
    }

    fn update_input_value(&self, v: &mut cynic_introspection::InputValue) {
        self.update_field_type(&mut v.ty);
    }

    fn update_field_type(&self, ty: &mut cynic_introspection::FieldType) {
        // We only want to update the type name if it isn't one of the built-in scalar types.
        if BUILTIN_SCALARS.contains(&ty.name.as_str()) {
            return;
        }

        self.prefixed(&mut ty.name);
    }

    fn add_field_resolvers(registry: &mut Registry) {
        for v in registry.types.values_mut() {
            let Some(i) = v.fields_mut() else { continue };

            for f in i.values_mut() {
                if f.resolver.is_parent() {
                    f.resolver = Resolver::Transformer(Transformer::GraphqlField);
                }
            }
        }
    }

    fn filter_builtins(schema: &mut cynic_introspection::Schema) {
        schema
            .directives
            .retain(|d| !BUILTIN_DIRECTIVES.contains(&d.name.as_str()));

        schema.types.retain(|t| {
            if let cynic_introspection::Type::Scalar(v) = &t {
                if BUILTIN_SCALARS.contains(&&*v.name) {
                    return false;
                }
            }

            // Filter out any types part of the introspection system.
            //
            // See: <http://spec.graphql.org/October2021/#sec-Names.Reserved-Names>
            return !t.name().starts_with("__");
        });
    }

    /// Add a new `Query` type with an `upstream` field to access the upstream API.
    fn add_root_query_field(&self, registry: &mut Registry, name: &str) {
        let root = registry
            .types
            .entry(registry.query_type.clone())
            .or_insert_with(|| ObjectType::new(registry.query_type.clone(), []).into());

        let Some(fields) = root.fields_mut() else { return };

        fields.insert(
            name.to_camel_case(),
            MetaField {
                name: name.to_camel_case(),
                description: Some(format!("Access to embedded {name} API.")),
                ty: format!("{}{}!", name.to_pascal_case(), &registry.query_type).into(),
                deprecation: Deprecation::NoDeprecated,
                cache_control: None,
                resolver: Resolver::Graphql(Box::new(graphql::Resolver::new(
                    self.name.clone(),
                    self.url.clone(),
                    Some(name.to_owned()),
                    self.type_prefix.clone(),
                ))),
                ..Default::default()
            },
        );
    }

    /// Add a new `Query` type with an `upstream` field to access the upstream API.
    fn update_root_query_fields(&self, registry: &mut Registry) {
        let root = registry
            .types
            .entry(registry.query_type.clone())
            .or_insert_with(|| ObjectType::new(registry.query_type.clone(), []).into());

        let Some(fields) = root.fields_mut() else { return };

        // There should always be fields for us to iterate, as we're mutating the `Query` object
        // fields from the upstream API. No fields, means no API access exposed by the upstream
        // server.
        for (_name, field) in fields {
            field.resolver = Resolver::Graphql(Box::new(graphql::Resolver::new(
                self.name.clone(),
                self.url.clone(),
                None,
                self.type_prefix.clone(),
            )));
        }
    }

    /// Add an optional `Mutate` type with an `upstream` field to access the upstream API.
    fn add_root_mutation_field(&self, registry: &mut Registry, name: &str) {
        let Some(mutation_type) = registry.mutation_type.clone() else {
            return;
        };

        let root = registry
            .types
            .entry(mutation_type.clone())
            .or_insert_with(|| ObjectType::new(mutation_type.clone(), []).into());

        let Some(fields) = root.fields_mut() else { return };

        fields.insert(
            name.to_camel_case(),
            MetaField {
                name: name.to_camel_case(),
                description: Some(format!("Access to embedded {name} API.")),
                ty: format!("{}{mutation_type}!", name.to_pascal_case()).into(),
                deprecation: Deprecation::NoDeprecated,
                cache_control: None,
                resolver: Resolver::Graphql(Box::new(graphql::Resolver::new(
                    self.name.clone(),
                    self.url.clone(),
                    Some(name.to_owned()),
                    self.type_prefix.clone(),
                ))),
                ..Default::default()
            },
        );
    }

    /// Add a new `Mutation` type with an `upstream` field to access the upstream API.
    fn update_root_mutation_fields(&self, registry: &mut Registry) {
        let Some(mutation_type) = registry.mutation_type.clone() else {
            return;
        };

        let root = registry
            .types
            .entry(mutation_type.clone())
            .or_insert_with(|| ObjectType::new(mutation_type.clone(), []).into());

        let Some(fields) = root.fields_mut() else { return };

        // There should always be fields for us to iterate, as we're mutating the `Mutation` object
        // fields from the upstream API. No fields, means no API access exposed by the upstream
        // server.
        for (_name, field) in fields {
            field.resolver = Resolver::Graphql(Box::new(graphql::Resolver::new(
                self.name.clone(),
                self.url.clone(),
                None,
                self.type_prefix.clone(),
            )));
        }
    }

    fn prefixed(&self, s: &mut String) {
        if let Some(prefix) = &self.type_prefix {
            *s = format!("{prefix}{s}");
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not complete request to GraphQL server: {0}")]
    HttpRequestError(#[from] CynicReqwestError),

    #[error("Could not parse the GraphQL inspection result: {0}")]
    JsonParsingError(#[from] serde_json::Error),

    #[error("Error returned when running introspection: {0}")]
    GraphqlError(#[from] GraphQlError),

    #[error("Could not parse the GraphQL schema: {0}")]
    SchemaError(#[from] SchemaError),

    #[error("Could not parse the HTTP headers: {0}")]
    HttpHeaderError(#[from] http::Error),

    #[error("Could not find valid data in GraphQL response")]
    MissingData,
}

#[derive(Clone, Debug)]
pub struct ApiMetadata {
    pub name: String,
    pub url: Url,
    pub headers: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use engine::registry::RegistrySdlExt;
    use registry_v2::ConnectorHeaderValue;
    use serde_json::json;
    use wiremock::{
        matchers::{header, method},
        Mock, MockServer, ResponseTemplate,
    };

    #[ctor::ctor]
    fn setup_rustls() {
        rustls::crypto::ring::default_provider().install_default().unwrap();
    }

    use super::*;

    #[tokio::test]
    async fn test_countries_output() {
        let introspection_headers = [
            ("x-client-id", "5ed1175bad06853b3aa1e492"),
            ("x-app-id", "623996f3c35130073829b252"),
        ];

        let data = include_str!("../tests/chargetrip_introspection.json");
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header(introspection_headers[0].0, introspection_headers[0].1))
            .and(header(introspection_headers[1].0, introspection_headers[1].1))
            .respond_with(ResponseTemplate::new(200).set_body_raw(data, "application/json"))
            .mount(&server)
            .await;

        let result = parse_schema(
            reqwest::Client::new(),
            "FooBar",
            true,
            &Url::parse(&server.uri()).unwrap(),
            ConnectorHeaders::new([]),
            introspection_headers,
            None,
        )
        .await
        .unwrap()
        .export_sdl(false);

        insta::assert_snapshot!(result);
    }

    #[tokio::test]
    async fn test_headers() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "__schema": {
                        "queryType": {"name":""},
                        "mutationType": {"name":""},
                        "subscriptionType": {"name":""},
                        "types": [],
                        "directives": [],
                    }
                }
            })))
            .mount(&server)
            .await;

        let headers = ConnectorHeaders::new([
            (
                "x-client-id".into(),
                ConnectorHeaderValue::Static("5ed1175bad06853b3aa1e492".into()),
            ),
            (
                "x-app-id".into(),
                ConnectorHeaderValue::Static("623996f3c35130073829b252".into()),
            ),
        ]);

        let result = parse_schema(
            reqwest::Client::new(),
            "FooBar",
            true,
            &Url::parse(&server.uri()).unwrap(),
            headers.clone(),
            std::iter::empty(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(
            result.http_headers,
            BTreeMap::from([(String::from("GraphQLConnectorFooBar"), headers)])
        );
    }

    #[tokio::test]
    async fn test_unnamed_connector() {
        let data = include_str!("../tests/chargetrip_introspection.json");
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(data, "application/json"))
            .mount(&server)
            .await;

        let result = parse_schema(
            reqwest::Client::new(),
            "Test",
            false,
            &Url::parse(&server.uri()).unwrap(),
            ConnectorHeaders::new([]),
            std::iter::empty(),
            None,
        )
        .await
        .unwrap()
        .export_sdl(false);

        insta::assert_snapshot!(result);
    }

    #[tokio::test]
    async fn test_custom_enum_values() {
        let data = include_str!("../tests/enum_value.json");
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(data, "application/json"))
            .mount(&server)
            .await;

        let result = parse_schema(
            reqwest::Client::new(),
            "Test",
            false,
            &Url::parse(&server.uri()).unwrap(),
            ConnectorHeaders::new([]),
            std::iter::empty(),
            None,
        )
        .await
        .unwrap()
        .export_sdl(false);

        insta::assert_snapshot!(result);
    }

    #[tokio::test]
    async fn test_preservation_of_type_casing() {
        let data = include_str!("../tests/casing.json");
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(data, "application/json"))
            .mount(&server)
            .await;

        let result = parse_schema(
            reqwest::Client::new(),
            "pre_fix",
            true,
            &Url::parse(&server.uri()).unwrap(),
            ConnectorHeaders::new([]),
            std::iter::empty(),
            None,
        )
        .await
        .unwrap()
        .export_sdl(false);

        insta::assert_snapshot!(result);
    }

    #[tokio::test]
    async fn test_contentful_schema() {
        let data = include_str!("../tests/contentful.json");
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(data, "application/json"))
            .mount(&server)
            .await;

        let mut registry = parse_schema(
            reqwest::Client::new(),
            "Contentful",
            true,
            &Url::parse(&server.uri()).unwrap(),
            ConnectorHeaders::new([]),
            std::iter::empty(),
            None,
        )
        .await
        .unwrap();

        registry.remove_unused_types();
        registry.remove_empty_types();

        let schema = registry.export_sdl(false);
        let diagnostics = graphql_schema_validation::validate(&schema);
        assert!(
            !diagnostics.has_errors(),
            "{:?}",
            diagnostics.iter().collect::<Vec<_>>()
        );
        insta::assert_snapshot!(schema);
    }
}
