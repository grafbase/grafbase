use dynaql::registry::{resolvers::http::ExpectedStatusCode, Registry};
use graph::OpenApiGraph;
use openapiv3::OpenAPI;
use parser::OpenApiQueryNamingStrategy as QueryNamingStrategy;
use parsing::components::Ref;
use url::Url;

mod graph;
mod output;
mod parsing;
mod validation;

pub fn parse_spec(
    data: &str,
    format: Format,
    mut metadata: ApiMetadata,
    registry: &mut Registry,
) -> Result<(), Vec<Error>> {
    // Make sure we have a trailing slash on metadata so that Url::join works correctly.
    ensure_trailing_slash(&mut metadata.url).map_err(|_| vec![Error::InvalidUrl(metadata.url.to_string())])?;

    let spec = match format {
        Format::Json => serde_json::from_str::<OpenAPI>(data).map_err(|e| vec![Error::JsonParsingError(e)])?,
        Format::Yaml => serde_yaml::from_str::<OpenAPI>(data).map_err(|e| vec![Error::YamlParsingError(e)])?,
    };

    let graph = OpenApiGraph::new(parsing::parse(spec)?, metadata.clone());

    validation::validate(&graph)?;

    output::output(&graph, registry);

    registry.http_headers.insert(metadata.name, metadata.headers);

    Ok(())
}

#[derive(Clone, Debug)]
pub struct ApiMetadata {
    pub name: String,
    pub url: Url,
    pub headers: Vec<(String, String)>,
    pub query_naming: QueryNamingStrategy,
}

impl From<parser::OpenApiDirective> for ApiMetadata {
    fn from(val: parser::OpenApiDirective) -> Self {
        ApiMetadata {
            name: val.name.clone(),
            url: val.url.clone(),
            headers: val.headers(),
            query_naming: val.transforms.query_naming,
        }
    }
}

pub enum Format {
    Json,
    Yaml,
}

impl Format {
    pub fn guess(content_type: Option<&str>, url: &str) -> Self {
        if let Some(content_type) = content_type {
            if content_type == "application/json" {
                return Format::Json;
            }
            if content_type.contains("yaml") {
                return Format::Yaml;
            }
        }
        if let Some(extension) = extract_extension(url) {
            if extension.eq_ignore_ascii_case("json") {
                return Format::Json;
            }
            if extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml") {
                return Format::Yaml;
            }
        }

        // YAML is a superset of JSON so lets just fallback to parsing as YAML.
        Format::Yaml
    }
}

fn extract_extension(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let last_segment = url.path_segments()?.last()?;

    let extension = std::path::Path::new(last_segment).extension()?;

    Some(extension.to_str()?.to_string())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not parse the open api spec: {0}")]
    JsonParsingError(serde_json::Error),
    #[error("Could not parse the open api spec: {0}")]
    YamlParsingError(serde_yaml::Error),
    #[error("The schema component {0} was a reference, which we don't currently support.")]
    TopLevelSchemaWasReference(String),
    #[error("The path component {0} was a reference, which we don't currently support.")]
    TopLevelPathWasReference(String),
    #[error("The response component {0} was a reference, which we don't currently support.")]
    TopLevelResponseWasReference(String),
    #[error("The request body component {0} was a reference, which we don't currently support.")]
    TopLevelRequestBodyWasReference(String),
    #[error("The parameter component {0} was a reference, which we don't currently support.")]
    TopLevelParameterWasReference(String),
    #[error("Couldn't parse HTTP method: {0}")]
    UnknownHttpMethod(String),
    #[error("An operation was marked with an unknown status code range: {0}")]
    UnknownStatusCodeRange(String),
    #[error("The operation {0} didn't have a response schema")]
    OperationMissingResponseSchema(String),
    #[error("The operation {0} didn't have a response schema")]
    OperationMissingRequestSchema(String),
    #[error("Encountered an array without items, which we don't currently support")]
    ArrayWithoutItems,
    #[error("Encountered a not schema, which we don't currently support")]
    NotSchema,
    #[error("Encountered an allOf schema, which we don't currently support")]
    AllOfSchema,
    #[error("Found a reference {0} which didn't seem to exist in the spec")]
    UnresolvedReference(Ref),
    #[error("Received an invalid URL: {0} ")]
    InvalidUrl(String),
    #[error("The path parameter {0} on operation {1} is an object, which is currently unsupported")]
    PathParameterIsObject(String, String),
    #[error("The path parameter {0} on operation {1} is a list, which is currently unsupported")]
    PathParameterIsList(String, String),
    #[error("The query parameter {0} on operation {1} is an object, which is currently unsupported")]
    QueryParameterIsObject(String, String),
    #[error("The query parameter {0} on operation {1} is a list, which is currently unsupported")]
    QueryParameterIsList(String, String),
    #[error("The query parameter {0} on operation {1} has a style {2}, which is currently unsupported")]
    UnsupportedQueryParameterStyle(String, String, String),
    #[error("The query parameter {0} on operation {1} has an object nested inside a list, which is unsupported")]
    ObjectNestedInsideListQueryParamter(String, String),
    #[error("The query parameter {0} on operation {1} has a non-scalar nested inside an object, which is unsupported")]
    NonScalarNestedInsideObjectQueryParameter(String, String),
    #[error("The query parameter {0} on operation {1} has a non-scalar nested inside an object, which is unsupported")]
    ListNestedInsideObjectQueryParameter(String, String),
}

fn is_ok(status: &ExpectedStatusCode) -> bool {
    match status {
        ExpectedStatusCode::Exact(200) => true,
        ExpectedStatusCode::Range(range) => *range == (200u16..300),
        _ => false,
    }
}

fn ensure_trailing_slash(url: &mut Url) -> Result<(), ()> {
    let mut segments = url.path_segments_mut()?;

    segments.pop_if_empty();
    segments.push("");

    Ok(())
}

#[cfg(test)]
mod tests {
    use dynaql::indexmap::IndexMap;

    use super::*;

    #[test]
    fn test_stripe_output() {
        insta::assert_snapshot!(
            build_registry("test_data/stripe.openapi.json", Format::Json, metadata("stripe")).export_sdl(false)
        );
    }

    #[test]
    fn test_petstore_output() {
        let registry = build_registry("test_data/petstore.openapi.json", Format::Json, metadata("petstore"));

        insta::assert_snapshot!(registry.export_sdl(false));
        insta::assert_debug_snapshot!(registry);
    }

    #[test]
    fn test_openai_output() {
        insta::assert_snapshot!(build_registry(
            "test_data/openai.yaml",
            Format::Yaml,
            ApiMetadata {
                query_naming: QueryNamingStrategy::OperationId,
                ..metadata("openai")
            }
        )
        .export_sdl(false));
    }

    #[test]
    fn test_impossible_unions() {
        insta::assert_snapshot!(
            build_registry("test_data/impossible-unions.json", Format::Json, metadata("petstore")).export_sdl(false)
        );
    }

    fn build_registry(schema_path: &str, format: Format, metadata: ApiMetadata) -> Registry {
        let mut registry = default_registry();

        parse_spec(
            &std::fs::read_to_string(schema_path).unwrap(),
            format,
            metadata,
            &mut registry,
        )
        .unwrap();

        registry
    }

    fn metadata(name: &str) -> ApiMetadata {
        ApiMetadata {
            name: name.into(),
            url: Url::parse("http://example.com").unwrap(),
            headers: vec![],
            query_naming: QueryNamingStrategy::SchemaName,
        }
    }

    fn default_registry() -> Registry {
        let mut registry = Registry {
            query_type: "Query".to_string(),
            ..Registry::default()
        };
        registry.types.insert(
            "Query".to_string(),
            dynaql::registry::MetaType::Object {
                name: "Query".to_string(),
                description: None,
                fields: IndexMap::new(),
                cache_control: Default::default(),
                extends: false,
                keys: None,
                visible: None,
                is_subscription: false,
                is_node: false,
                rust_typename: "Query".to_string(),
                constraints: vec![],
            },
        );

        registry
    }
}
