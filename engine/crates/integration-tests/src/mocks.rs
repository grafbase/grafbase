use async_trait::async_trait;
use parser_sdl::{ConnectorParsers, GraphqlDirective, NeonDirective, OpenApiDirective, Registry};
use postgresql_types::transport::NeonTransport;

pub struct MockConnectorParsers;

#[async_trait]
impl ConnectorParsers for MockConnectorParsers {
    async fn fetch_and_parse_openapi(&self, _: OpenApiDirective) -> Result<Registry, Vec<String>> {
        Ok(Registry::new())
    }

    async fn fetch_and_parse_graphql(&self, _: GraphqlDirective) -> Result<Registry, Vec<String>> {
        Ok(Registry::new())
    }

    async fn fetch_and_parse_neon(&self, directive: &NeonDirective) -> Result<Registry, Vec<String>> {
        let transport = NeonTransport::new(directive.postgresql_url()).map_err(|error| vec![error.to_string()])?;

        parser_postgresql::introspect(&transport, directive.name(), directive.namespace())
            .await
            .map_err(|error| vec![error.to_string()])
    }
}
