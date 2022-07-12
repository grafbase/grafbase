use crate::parser::types::ConstDirective;
use crate::{ServerError, Value};

use dynaql_value::ConstValue;

const OIDC_PROVIDER: &str = "oidc";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub providers: Vec<AuthProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthProvider {
    pub r#type: String, // TODO: turn this into an enum once we support more providers
    pub issuer: url::Url,
}

impl Auth {
    pub fn oidc_provider(&self) -> Option<&AuthProvider> {
        self.providers.iter().find(|p| p.r#type == OIDC_PROVIDER)
    }
}

impl TryFrom<&ConstDirective> for Auth {
    type Error = ServerError;

    fn try_from(value: &ConstDirective) -> Result<Self, Self::Error> {
        let pos = Some(value.name.pos);

        let arg = match value.get_argument("providers") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) => value,
                _ => return Err(ServerError::new("auth providers must be a list", pos)),
            },
            None => return Err(ServerError::new("auth providers missing", pos)),
        };

        let providers = arg
            .iter()
            .map(AuthProvider::try_from)
            .collect::<Result<_, _>>()
            .map_err(|err| ServerError::new(err.message, pos))?;

        Ok(Auth { providers })
    }
}

impl TryFrom<&ConstValue> for AuthProvider {
    type Error = ServerError;

    fn try_from(value: &ConstValue) -> Result<Self, Self::Error> {
        let provider = match value {
            ConstValue::Object(value) => value,
            _ => return Err(ServerError::new("auth provider must be an object", None)),
        };

        let typ = match provider.get("type") {
            Some(Value::String(value)) => value.to_string(),
            _ => return Err(ServerError::new("auth provider: type missing", None)),
        };
        if typ != OIDC_PROVIDER {
            return Err(ServerError::new(
                format!("auth provider: type must be `{OIDC_PROVIDER}`"),
                None,
            ));
        }

        let issuer = match provider.get("issuer") {
            Some(Value::String(value)) => match value.parse() {
                Ok(url) => url,
                Err(_) => return Err(ServerError::new("auth provider: invalid issuer URL", None)),
            },
            _ => return Err(ServerError::new("auth provider: issuer missing", None)),
        };

        Ok(AuthProvider {
            r#type: typ,
            issuer,
        })
    }
}
