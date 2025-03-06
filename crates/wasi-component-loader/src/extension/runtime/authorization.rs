use runtime::error::PartialErrorCode;

use crate::extension::wit;

impl From<wit::AuthorizationDecisions> for runtime::extension::AuthorizationDecisions {
    fn from(decisions: wit::AuthorizationDecisions) -> Self {
        match decisions {
            wit::AuthorizationDecisions::GrantAll => runtime::extension::AuthorizationDecisions::GrantAll,
            wit::AuthorizationDecisions::DenyAll(error) => runtime::extension::AuthorizationDecisions::DenyAll(
                error.into_graphql_error(PartialErrorCode::Unauthorized),
            ),
            wit::AuthorizationDecisions::DenySome(wit::AuthorizationDecisionsDenySome {
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
