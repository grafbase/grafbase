use async_trait::async_trait;
use sdl_parser::{ConnectorParsers, GraphqlDirective, OpenApiDirective, Registry};

pub struct MockConnectorParsers;

#[async_trait]
impl ConnectorParsers for MockConnectorParsers {
    async fn fetch_and_parse_openapi(&self, _: OpenApiDirective) -> Result<Registry, Vec<String>> {
        Ok(Registry::new())
    }

    async fn fetch_and_parse_graphql(&self, _: GraphqlDirective) -> Result<Registry, Vec<String>> {
        Ok(Registry::new())
    }
}
