use engine_error::ErrorCode;

use crate::state::InstanceState;

pub use super::grafbase::sdk::authorization_types::*;

impl Host for InstanceState {}

impl From<AuthorizationDecisions> for runtime::extension::AuthorizationDecisions {
    fn from(decisions: AuthorizationDecisions) -> Self {
        match decisions {
            AuthorizationDecisions::GrantAll => runtime::extension::AuthorizationDecisions::GrantAll,
            AuthorizationDecisions::DenyAll(error) => {
                runtime::extension::AuthorizationDecisions::DenyAll(error.into_graphql_error(ErrorCode::Unauthorized))
            }
            AuthorizationDecisions::DenySome(AuthorizationDecisionsDenySome {
                element_to_error,
                errors,
            }) => {
                let errors = errors
                    .into_iter()
                    .map(|err| err.into_graphql_error(ErrorCode::Unauthorized))
                    .collect();

                runtime::extension::AuthorizationDecisions::DenySome {
                    element_to_error,
                    errors,
                }
            }
        }
    }
}
