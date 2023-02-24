use dynaql::registry::Registry;
use graph::OpenApiGraph;
use openapiv3::OpenAPI;
use parsing::components::Ref;
use url::Url;

mod graph;
mod output;
mod parsing;

pub fn parse_spec(
    data: &str,
    format: Format,
    metadata: ApiMetadata,
    registry: &mut Registry,
) -> Result<(), Vec<Error>> {
    let spec = match format {
        Format::Json => serde_json::from_str::<OpenAPI>(data).map_err(|e| vec![Error::JsonParsingError(e)])?,
        Format::Yaml => serde_yaml::from_str::<OpenAPI>(data).map_err(|e| vec![Error::YamlParsingError(e)])?,
    };

    let graph = OpenApiGraph::new(parsing::parse(spec)?, metadata);

    output::output(&graph, registry);

    Ok(())
}

pub struct ApiMetadata {
    pub name: String,
    pub url: Url,
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
    #[error("Couldn't parse HTTP verb: {0}")]
    UnknownHttpVerb(String),
    #[error("The operation {0} didn't have a response schema")]
    OperationMissingResponseSchema(String),
    #[error("Encountered an array without items, which we don't currently support")]
    ArrayWithoutItems,
    #[error("Encountered a not schema, which we don't currently support")]
    NotSchema,
    #[error("Encountered an allOf schema, which we don't currently support")]
    AllOfSchema,
    #[error("Encountered an any schema, which we don't currently support")]
    AnySchema,
    #[error("Found a reference {0} which didn't seem to exist in the spec")]
    UnresolvedReference(Ref),
}

fn is_ok(status: &openapiv3::StatusCode) -> bool {
    match status {
        openapiv3::StatusCode::Code(200) => true,
        openapiv3::StatusCode::Range(_range) => todo!(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use dynaql::{indexmap::IndexMap, CacheControl};

    use super::*;

    #[test]
    fn test_stripe_output() {
        let spec = std::fs::read_to_string("test_data/stripe.openapi.json").unwrap();

        let mut registry = default_registry();

        parse_spec(&spec, Format::Json, metadata(), &mut registry).unwrap();

        insta::assert_snapshot!(registry.export_sdl(false));
    }

    fn metadata() -> ApiMetadata {
        ApiMetadata {
            name: "example".into(),
            url: Url::parse("http://example.com").unwrap(),
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
                cache_control: CacheControl {
                    public: true,
                    max_age: 0,
                },
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
