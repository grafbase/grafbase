use crate::parser::types::ConstDirective;
use crate::{ServerError, Value};

use dynaql_value::ConstValue;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub providers: Vec<AuthProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthProvider {
    // TODO: add type
    pub issuer: url::Url,
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
        let value = match value {
            ConstValue::Object(value) => value,
            _ => return Err(ServerError::new("auth provider must be an object", None)),
        };

        let issuer = match value.get("issuer") {
            Some(Value::String(value)) => match value.parse() {
                Ok(url) => url,
                Err(_) => return Err(ServerError::new("auth provider: invalid issuer URL", None)),
            },
            _ => return Err(ServerError::new("auth provider: issuer missing", None)),
        };

        Ok(AuthProvider { issuer })
    }
}
