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

        client_id: Option<DynamicString>,
    },

    #[serde(rename_all = "camelCase")]
    Jwks {
        issuer: DynamicString,

        jwks_endpoint: Option<DynamicString>,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<DynamicString>,
    },

    #[serde(rename_all = "camelCase")]
    Jwt {
        issuer: DynamicString,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<DynamicString>,

        secret: DynamicString,
    },
}

fn default_groups_claim() -> String {
    DEFAULT_GROUPS_CLAIM.to_string()
}

impl AuthProvider {
    fn validate_url(dynamic_string: &DynamicString, error_prefix: &'static str) -> Result<(), ServerError> {
        dynamic_string
            .as_fully_evaluated_str()
            .map(|s| s.parse::<url::Url>())
            .transpose()
            .map_err(|err| {
                // FIXME: Pass in the proper location here and everywhere above as it's not done properly now.
                ServerError::new(format!("{error_prefix}: {err}"), None)
            })
            .map(|_| ())
    }

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
            AuthProvider::Oidc {
                ref mut issuer,
                ref mut client_id,
                ..
            } => {
                ctx.partially_evaluate_literal(issuer)?;
                Self::validate_url(issuer, "OIDC provider")?;

                if let Some(client_id) = client_id {
                    ctx.partially_evaluate_literal(client_id)?;
                }
            }
            AuthProvider::Jwks {
                ref mut issuer,
                ref mut jwks_endpoint,
                ref mut client_id,
                ..
            } => {
                ctx.partially_evaluate_literal(issuer)?;

                if let Some(jwks_endpoint) = jwks_endpoint {
                    ctx.partially_evaluate_literal(jwks_endpoint)?;
                    Self::validate_url(jwks_endpoint, "JWKS provider")?;
                } else {
                    // issuer must be a URL in this case
                    Self::validate_url(issuer, "JWKS provider")?;
                }

                if let Some(client_id) = client_id {
                    ctx.partially_evaluate_literal(client_id)?;
                }
            }

            AuthProvider::Jwt {
                ref mut issuer,
                ref mut secret,
                ref mut client_id,
                ..
            } => {
                ctx.partially_evaluate_literal(issuer)?;

                if let Some(client_id) = client_id {
                    ctx.partially_evaluate_literal(client_id)?;
                }

                ctx.partially_evaluate_literal(secret)?;
            }
        }

        Ok(provider)
    }
}
