use engine::ServerError;
use serde::{Deserialize, Serialize};

pub const DEFAULT_GROUPS_CLAIM: &str = "groups";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum AuthProvider {
    #[serde(rename_all = "camelCase")]
    Oidc {
        issuer: String,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    Jwks {
        // at least one of issuer, jwks_endpoint must be set
        issuer: Option<String>,

        jwks_endpoint: Option<String>,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    Jwt {
        issuer: String,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,

        client_id: Option<String>,

        secret: String,
    },

    #[serde(rename_all = "camelCase")]
    Authorizer { name: String },
}

fn default_groups_claim() -> String {
    DEFAULT_GROUPS_CLAIM.to_string()
}

impl AuthProvider {
    fn validate_url(str: &str, error_prefix: &'static str) -> Result<url::Url, ServerError> {
        str.parse::<url::Url>().map_err(|err| {
            // FIXME: Pass in the proper location here and everywhere above as it's not done properly now.
            ServerError::new(format!("{error_prefix}: {err}"), None)
        })
    }

    pub fn validate(mut self) -> Result<Self, ServerError> {
        match self {
            AuthProvider::Oidc { ref mut issuer, .. } => {
                Self::validate_url(issuer, "OIDC provider")?;
            }
            AuthProvider::Jwks {
                ref mut issuer,
                ref mut jwks_endpoint,
                ..
            } => {
                match (issuer, jwks_endpoint.as_mut()) {
                    (None, None) => Err(ServerError::new(
                        "JWKS provider: at least one of 'issuer', 'jwks_endpoint' must be set.".to_string(),
                        None,
                    )),
                    (Some(issuer), None) => {
                        // issuer must be a URL in this case so that jwks_endpoint can be constructed.
                        let mut url = Self::validate_url(issuer, "JWKS provider")?;
                        const JWKS_PATH_SEGMENTS: [&str; 2] = [".well-known", "jwks.json"];

                        url.path_segments_mut()
                            .map_err(|_| {
                                ServerError::new(
                                    String::from("JWKS provider: encountered a cannot-be-a-base issuer url"),
                                    None,
                                )
                            })?
                            .extend(JWKS_PATH_SEGMENTS);

                        *jwks_endpoint = Some(url.to_string());
                        Ok(())
                    }
                    (_, Some(jwks_endpoint)) => Self::validate_url(jwks_endpoint, "JWKS provider").map(|_| ()),
                }?;
            }
            AuthProvider::Jwt { .. } | AuthProvider::Authorizer { .. } => {}
        }
        Ok(self)
    }
}
