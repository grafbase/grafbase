use serde::{Deserialize, Serialize};

use dynaql::ServerError;
use dynaql_value::ConstValue;

use crate::dynamic_string::DynamicString;
use crate::VisitorContext;

pub const DEFAULT_GROUPS_CLAIM: &str = "groups";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum AuthProvider {
    #[serde(rename_all = "camelCase")]
    Oidc {
        issuer: DynamicString,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    Jwt {
        issuer: DynamicString,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<String>,

        secret: DynamicString,
    },
}

fn default_groups_claim() -> String {
    DEFAULT_GROUPS_CLAIM.to_string()
}

impl AuthProvider {
    pub fn from_value(ctx: &VisitorContext<'_>, value: &ConstValue) -> Result<Self, ServerError> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth provider must be an object", None)),
        };

        let mut provider: AuthProvider =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth provider: {err}"), None))?;

        match provider {
            AuthProvider::Oidc { ref mut issuer, .. } => {
                ctx.partially_evaluate_literal(issuer)?;
                if let Err(err) = issuer
                    .as_fully_evaluated_str()
                    .map(|s| s.parse::<url::Url>())
                    .transpose()
                {
                    // FIXME: Pass in the proper location here and everywhere above as it's not done properly now.
                    return Err(ServerError::new(format!("OIDC provider: {err}"), None));
                }
            }
            AuthProvider::Jwt {
                ref mut issuer,
                ref mut secret,
                ..
            } => {
                ctx.partially_evaluate_literal(issuer)?;
                if let Err(err) = issuer
                    .as_fully_evaluated_str()
                    .map(|s| s.parse::<url::Url>())
                    .transpose()
                {
                    return Err(ServerError::new(format!("JWT provider: {err}"), None));
                }

                ctx.partially_evaluate_literal(secret)?;
            }
        }

        Ok(provider)
    }
}
