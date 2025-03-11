use runtime::error::PartialErrorCode;

use crate::extension::api::wit::authorization as wit_latest;

impl From<wit_latest::AuthorizationDecisions> for runtime::extension::AuthorizationDecisions {
    fn from(decisions: wit_latest::AuthorizationDecisions) -> Self {
        match decisions {
            wit_latest::AuthorizationDecisions::GrantAll => runtime::extension::AuthorizationDecisions::GrantAll,
            wit_latest::AuthorizationDecisions::DenyAll(error) => runtime::extension::AuthorizationDecisions::DenyAll(
                error.into_graphql_error(PartialErrorCode::Unauthorized),
            ),
            wit_latest::AuthorizationDecisions::DenySome(wit_latest::AuthorizationDecisionsDenySome {
                element_to_error,
                errors,
            }) => {
                let errors = errors
                    .into_iter()
                    .map(|err| err.into_graphql_error(PartialErrorCode::Unauthorized))
                    .collect();

                runtime::extension::AuthorizationDecisions::DenySome {
                    element_to_error,
                    errors,
                }
            }
        }
    }
}
