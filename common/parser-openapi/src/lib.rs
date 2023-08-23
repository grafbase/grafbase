use grafbase_engine::registry::{resolvers::http::ExpectedStatusCode, ConnectorHeaders, Registry};
use graph::OpenApiGraph;
use inflector::Inflector;
use sdl_parser::OpenApiQueryNamingStrategy as QueryNamingStrategy;
use tracing as _;
use url::Url;

mod graph;
mod output;
mod parsing;
mod validation;

pub fn parse_spec(
    data: String,
    format: Format,
    mut metadata: ApiMetadata,
    registry: &mut Registry,
) -> Result<(), Vec<Error>> {
    let parsed = parsing::parse(data, format)?;

    if metadata.url.is_none() {
        metadata.url = Some(parsed.url.clone().map_err(|error| vec![error])?);
    }

    let url = metadata.url.as_mut().unwrap();

    // Make sure we have a trailing slash on metadata so that Url::join works correctly.
    ensure_trailing_slash(url).map_err(|_| vec![Error::InvalidUrl(url.to_string())])?;

    let graph = OpenApiGraph::new(parsed, metadata.clone()).map_err(|error| vec![error])?;

    validation::validate(&graph)?;

    output::output(&graph, registry);

    registry
        .http_headers
        .insert(metadata.unique_namespace(), metadata.headers);

    Ok(())
}

#[derive(Clone, Debug)]
pub struct ApiMetadata {
    pub name: String,
    pub namespace: bool,
    pub url: Option<Url>,
    pub headers: ConnectorHeaders,
    pub query_naming: QueryNamingStrategy,
}

impl ApiMetadata {
    pub fn unique_namespace(&self) -> String {
        self.name.to_camel_case()
    }

    pub fn namespaced(&self, name: &str) -> String {
        format!("{}_{}", self.unique_namespace(), name)
    }
}

impl From<sdl_parser::OpenApiDirective> for ApiMetadata {
    fn from(val: sdl_parser::OpenApiDirective) -> Self {
        let headers = val.headers();

        ApiMetadata {
            name: val.name,
            namespace: val.namespace,
            url: val.url,
            headers,
            query_naming: val.transforms.query_naming,
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("We don't support version {0} of OpenAPI currently.  Please contact support")]
    UnsupportedVersion(String),
    #[error("There was no URL in this OpenAPI document.  Please provide the url parameter to `@openapi`")]
    MissingUrl,
    #[error("Could not parse the open api spec: {0}")]
    JsonParsingError(String),
    #[error("Could not parse the open api spec: {0}")]
    YamlParsingError(String),
    #[error("The schema component {0} was a reference, which we don't currently support.")]
    TopLevelSchemaWasReference(String),
    #[error("The schema component {0} was a boolean, which we don't currently support.")]
    TopLevelSchemaWasBoolean(String),
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
    #[error("Encountered an array with a list of items, which we don't currently support")]
    ArrayWithManyItems,
    #[error("Encountered a not schema, which we don't currently support")]
    NotSchema,
    #[error("Found a reference {0} which didn't seem to exist in the spec")]
    UnresolvedReference(String),
    #[error("We couldn't parse the URL: `{0}`  You might need to provide or fix the url parameter to `@openapi`")]
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
    #[error("We found a cycle of allOf objects in the OpenAPI schema, which is unsupported")]
    AllOfCycle,
}

fn is_ok(status: &ExpectedStatusCode) -> bool {
    match status {
        ExpectedStatusCode::Exact(200) => true,
        ExpectedStatusCode::Exact(other) if (200..300).contains(other) => true,
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
    use assert_matches::assert_matches;
    use grafbase_engine::registry::{MetaType, UnionType};

    use super::*;

    #[test]
    fn test_stripe_output() {
        let metadata = ApiMetadata {
            url: None,
            ..metadata("stripe", true)
        };
        insta::assert_snapshot!(build_registry("test_data/stripe.openapi.json", Format::Json, metadata)
            .unwrap()
            .export_sdl(false));
    }

    #[test]
    fn test_stripe_output_json() {
        let metadata = ApiMetadata {
            url: None,
            ..metadata("stripe", true)
        };
        let registry = build_registry("test_data/stripe.openapi.json", Format::Json, metadata).unwrap();
        insta::assert_json_snapshot!(registry);
    }

    #[test]
    fn test_petstore_output() {
        let registry = build_registry(
            "test_data/petstore.openapi.json",
            Format::Json,
            metadata("petstore", true),
        )
        .unwrap();

        insta::assert_snapshot!(registry.export_sdl(false));
        insta::assert_debug_snapshot!(registry);
    }

    #[test]
    fn test_flat_output() {
        let registry = build_registry(
            "test_data/petstore.openapi.json",
            Format::Json,
            metadata("petstore", false),
        )
        .unwrap();

        insta::assert_snapshot!(registry.export_sdl(false));
        insta::assert_debug_snapshot!(registry);
    }

    #[test]
    fn test_url_without_host_failure() {
        let metadata = ApiMetadata {
            url: None,
            ..metadata("petstore", true)
        };
        assert_matches!(
            build_registry("test_data/petstore.openapi.json", Format::Json, metadata)
                .unwrap_err()
                .as_slice(),
            [Error::InvalidUrl(url)] => {
                assert_eq!(url, "/api/v3");
            }
        );
    }

    #[test]
    fn test_openai_output() {
        insta::assert_snapshot!(build_registry(
            "test_data/openai.yaml",
            Format::Yaml,
            ApiMetadata {
                query_naming: QueryNamingStrategy::OperationId,
                ..metadata("openai", true)
            }
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_planetscale() {
        insta::assert_snapshot!(build_registry(
            "test_data/planetscale.json",
            Format::Json,
            ApiMetadata {
                url: None,
                ..metadata("planetscale", true)
            }
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_orb() {
        // Orb is a 3.1 spec
        insta::assert_snapshot!(build_registry(
            "test_data/orb.json",
            Format::Json,
            ApiMetadata {
                url: None,
                ..metadata("orb", true)
            }
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_mongo_atlas() {
        tracing_subscriber::fmt().with_env_filter("trace").pretty().init();

        // Mongo Atlas is a 3.1 spec
        insta::assert_snapshot!(build_registry(
            "test_data/mongo-atlas.json",
            Format::Json,
            ApiMetadata {
                url: None,
                ..metadata("mongo", true)
            }
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_impossible_unions() {
        insta::assert_snapshot!(build_registry(
            "test_data/impossible-unions.json",
            Format::Json,
            metadata("petstore", true)
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_all_of_schema_simple() {
        // an allOf schema that are simple merges of distinct objects
        insta::assert_snapshot!(build_registry(
            "test_data/all-ofs-simple.json",
            Format::Json,
            metadata("petstore", true)
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_all_of_schema_complex() {
        // Some allOf schemas have properties defined in multiple branches
        // of the allOf, with required sometiems being in one branch but not
        // another.  This is a test of that....
        insta::assert_snapshot!(build_registry(
            "test_data/all-ofs-complex.json",
            Format::Json,
            metadata("petstore", true)
        )
        .unwrap()
        .export_sdl(false));
    }

    #[test]
    fn test_supabase() {
        insta::assert_snapshot!(
            build_registry("test_data/supabase.json", Format::Json, metadata("supabase", true))
                .unwrap()
                .export_sdl(false)
        );
    }

    #[test]
    fn test_stripe_discrimnator_detection() {
        let registry = build_registry("test_data/stripe.openapi.json", Format::Json, metadata("stripe", true)).unwrap();
        let discriminators = registry
            .types
            .values()
            .filter_map(|ty| match ty {
                MetaType::Union(UnionType {
                    name, discriminators, ..
                }) => Some((name, discriminators)),
                _ => None,
            })
            .collect::<Vec<_>>();

        insta::assert_json_snapshot!(discriminators);
    }

    fn build_registry(schema_path: &str, format: Format, metadata: ApiMetadata) -> Result<Registry, Vec<Error>> {
        let mut registry = Registry::new();

        parse_spec(
            std::fs::read_to_string(schema_path).unwrap(),
            format,
            metadata,
            &mut registry,
        )?;

        Ok(registry)
    }

    fn metadata(name: &str, namespace: bool) -> ApiMetadata {
        ApiMetadata {
            name: name.to_string(),
            namespace,
            url: Some(Url::parse("http://example.com").unwrap()),
            headers: ConnectorHeaders::new([]),
            query_naming: QueryNamingStrategy::SchemaName,
        }
    }
}
