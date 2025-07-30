use ahash::HashMap;
use grafbase_sdk::{
    AuthorizationExtension, IntoQueryAuthorization,
    host_io::{
        self,
        http::{HttpRequest, Url},
    },
    types::{AuthorizationDecisions, Configuration, Error, ErrorResponse, QueryElements, SubgraphHeaders, Token},
};

const ERROR_MESSAGE: &str = "Not authorized: policy not granted.";

#[derive(AuthorizationExtension)]
struct PolicyExtension {
    policy_url: Url,
}

#[derive(serde::Deserialize)]
struct Config {
    auth_service_url: String,
}

#[derive(serde::Deserialize)]
struct Policy<'a> {
    #[serde(borrow)]
    policies: Vec<Vec<&'a str>>,
}

impl AuthorizationExtension for PolicyExtension {
    fn new(config: Configuration) -> Result<Self, Error> {
        let Config { mut auth_service_url } = config.deserialize()?;
        auth_service_url.push_str("/policy");
        let policy_url = auth_service_url
            .parse()
            .map_err(|err| format!("Invalid Policy URL: {err}"))?;
        Ok(Self { policy_url })
    }

    fn authorize_query(
        &mut self,
        _headers: &mut SubgraphHeaders,
        _token: Token,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoQueryAuthorization, ErrorResponse> {
        //
        // Accumulate a list of all policies we need to validate
        //
        let mut policies_to_grant = HashMap::default();
        let mut element_to_policies = Vec::with_capacity(elements.len());

        for element in elements.iter() {
            let Policy { policies } = element.directive_arguments::<Policy>()?;
            for nested in &policies {
                for p in nested {
                    policies_to_grant.insert(*p, false);
                }
            }
            element_to_policies.push(policies);
        }

        // Insert your custom logic here
        let granted_policies = self
            .grant_policies(policies_to_grant)
            .map_err(|err| ErrorResponse::unauthorized().with_error(err))?;

        //
        // Process the query elements now that we have the granted policies
        //
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let mut lazy_error_id = None;

        for (element, policies) in elements.into_iter().zip(element_to_policies) {
            let has_matching_scopes = policies
                .into_iter()
                .any(|policies_and| policies_and.into_iter().all(|policy| granted_policies[policy]));

            if !has_matching_scopes {
                // We only need to show the error once, so we re-use the same error.
                let error_id = *lazy_error_id.get_or_insert_with(|| builder.push_error(ERROR_MESSAGE));
                builder.deny_with_error_id(element, error_id);
            }
        }

        Ok(builder.build())
    }
}

impl PolicyExtension {
    // Executes the HTTP request to the policy service to check which policies are granted.
    fn grant_policies<'a>(
        &mut self,
        mut policies_to_grant: HashMap<&'a str, bool>,
    ) -> Result<HashMap<&'a str, bool>, Error> {
        #[derive(serde::Serialize)]
        struct Request<'a> {
            policies: Vec<&'a str>,
        }

        #[derive(serde::Deserialize)]
        struct Response {
            granted: Vec<bool>,
        }

        let request = Request {
            policies: policies_to_grant.keys().cloned().collect(),
        };
        let response =
            host_io::http::execute(HttpRequest::post(self.policy_url.clone()).json(&request)).map_err(|err| {
                log::error!("Failed to fetch policies: {err}");
                ERROR_MESSAGE
            })?;

        let Response { granted } = response.json().map_err(|err| {
            log::error!("Failed to parse policy response: {err}");
            ERROR_MESSAGE
        })?;
        for (policy, granted) in request.policies.into_iter().zip(granted) {
            policies_to_grant.insert(policy, granted);
        }

        Ok(policies_to_grant)
    }
}
