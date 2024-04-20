use std::collections::{HashMap, HashSet};

use common_types::UdfKind;
use engine::registry::Registry;
use itertools::Itertools;
use parser_sdl::{GraphqlDirective, OpenApiDirective, ParseResult, PostgresDirective};
use postgres_connector_types::transport::DirectTcpTransport;

use super::ConfigError;

// Contract between this crate and CLI
// #[derive(serde::Serialize)]
pub struct ParserResult {
    pub registry: Registry,
    pub required_udfs: HashSet<(UdfKind, String)>,
    pub federated_graph_config: Option<parser_sdl::federation::FederatedGraphConfig>,
}

/// Transform the input schema into a Registry
pub async fn parse_sdl(schema: &str, environment: &HashMap<String, String>) -> Result<ParserResult, ConfigError> {
    let connector_parsers = ConnectorParsers {
        http_client: reqwest::Client::new(),
    };

    let parse = parser_sdl::parse(schema, environment, &connector_parsers)
        .await
        .map_err(|e| ConfigError::ParseSchema(e.to_string()))?;

    std::fs::write("parse-result.json", serde_json::to_vec(&parse).unwrap()).unwrap();

    let ParseResult {
        mut registry,
        required_udfs,
        global_cache_rules,
        federated_graph_config,
    } = parse;

    // apply global caching rules
    global_cache_rules
        .apply(&mut registry)
        .map_err(|e| ConfigError::ParseSchema(e.into_iter().join("\n")))?;

    Ok(ParserResult {
        registry,
        required_udfs,
        federated_graph_config,
    })
}

struct ConnectorParsers {
    http_client: reqwest::Client,
}

#[async_trait::async_trait]
impl parser_sdl::ConnectorParsers for ConnectorParsers {
    async fn fetch_and_parse_openapi(&self, directive: OpenApiDirective) -> Result<Registry, Vec<String>> {
        let mut request = self.http_client.get(&directive.schema_url);

        for (name, value) in directive.introspection_headers() {
            request = request.header(name, value);
        }

        let response = request.send().await.map_err(|e| vec![e.to_string()])?;

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|header_value| header_value.to_str().ok())
            .map(ToOwned::to_owned);

        let spec = response.text().await.map_err(|e| vec![e.to_string()])?;

        let format = parser_openapi::Format::guess(content_type.as_deref(), &directive.schema_url);

        let mut registry = Registry::new();

        parser_openapi::parse_spec(spec, format, directive.into(), &mut registry)
            .map_err(|errors| errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>())?;

        Ok(registry)
    }

    async fn fetch_and_parse_graphql(&self, directive: GraphqlDirective) -> Result<Registry, Vec<String>> {
        parser_graphql::parse_schema(
            self.http_client.clone(),
            &directive.name,
            directive.namespace,
            &directive.url,
            directive.headers(),
            directive.introspection_headers(),
            directive
                .transforms
                .as_ref()
                .and_then(|transforms| transforms.prefix_types.as_deref()),
        )
        .await
        .map_err(|errors| errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>())
    }

    async fn fetch_and_parse_postgres(&self, directive: &PostgresDirective) -> Result<Registry, Vec<String>> {
        let transport = DirectTcpTransport::new(directive.connection_string())
            .await
            .map_err(|error| vec![error.to_string()])?;

        parser_postgres::introspect(&transport, directive.name(), directive.namespace())
            .await
            .map_err(|error| vec![error.to_string()])
    }
}
