use grafbase_sdk::{
    AuthorizationExtension, IntoQueryAuthorization,
    types::{AuthorizationDecisions, Configuration, Error, ErrorResponse, QueryElements, SubgraphHeaders, Token},
};

#[derive(AuthorizationExtension)]
struct Authz19SubgraphGrouping {
    config: Config,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub denied_subgraph_names: Vec<String>,
}

impl AuthorizationExtension for Authz19SubgraphGrouping {
    fn new(config: Configuration) -> Result<Self, Error> {
        Ok(Self {
            config: config.deserialize()?,
        })
    }

    fn authorize_query(
        &mut self,
        _headers: &mut SubgraphHeaders,
        _token: Token,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoQueryAuthorization, ErrorResponse> {
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let error_id = builder.push_error(Error::new("Not authorized, denied subgraph SDK19"));

        for element in elements {
            let subgraph_name = element.subgraph_name().expect("Missing subgraph name");
            let denied = self.config.denied_subgraph_names.iter().any(|s| s == subgraph_name);
            log::info!(
                "{} -> {} = {}",
                subgraph_name,
                match element.directive_site() {
                    grafbase_sdk::types::DirectiveSite::Object(site) => site.name(),
                    grafbase_sdk::types::DirectiveSite::FieldDefinition(site) => site.name(),
                    grafbase_sdk::types::DirectiveSite::Interface(site) => site.name(),
                    grafbase_sdk::types::DirectiveSite::Union(site) => site.name(),
                    grafbase_sdk::types::DirectiveSite::Enum(site) => site.name(),
                    grafbase_sdk::types::DirectiveSite::Scalar(site) => site.name(),
                },
                denied
            );
            if denied {
                builder.deny_with_error_id(element, error_id);
            }
        }

        Ok(builder.build())
    }
}
