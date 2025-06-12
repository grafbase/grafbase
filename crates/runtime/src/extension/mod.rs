mod authentication;
mod authorization;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;

pub use authentication::*;
pub use authorization::*;
use bytes::Bytes;
use error::GraphqlError;
pub use field_resolver::*;
pub use hooks::*;
pub use resolver::*;
pub use selection_set_resolver::*;

pub trait ExtensionRuntime:
    AuthenticationExtension<Self::Context>
    + AuthorizationExtension<Self::Context>
    + FieldResolverExtension<Self::Context>
    + SelectionSetResolverExtension
    + ResolverExtension<Self::Context>
    + Send
    + Sync
    + 'static
{
    type Context: Send + Sync + 'static;
}

#[derive(Debug, Clone)]
pub struct Response {
    pub data: Option<Data>,
    pub errors: Vec<GraphqlError>,
}

impl Response {
    pub fn data(data: Data) -> Self {
        Self {
            data: Some(data),
            errors: Vec::new(),
        }
    }

    pub fn error(err: GraphqlError) -> Self {
        Self {
            data: None,
            errors: vec![err],
        }
    }

    // For legacy resolver SDKs
    pub fn legacy_into_result(self) -> Result<Data, GraphqlError> {
        if let Some(err) = self.errors.into_iter().next() {
            Err(err)
        } else if let Some(data) = self.data {
            Ok(data)
        } else {
            Ok(Data::Json(Default::default()))
        }
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data: serde_json::Value = match &self.data {
            Some(Data::Json(bytes)) => serde_json::from_slice(bytes).unwrap_or("<error>".into()),
            Some(Data::Cbor(bytes)) => minicbor_serde::from_slice(bytes).unwrap_or("<error>".into()),
            None => Default::default(),
        };
        let errors = self
            .errors
            .iter()
            .map(|err| {
                serde_json::json!({
                    "message": err.message.clone(),
                    "extensions": err.extensions.clone(),
                })
            })
            .collect::<Vec<_>>();
        let resp = serde_json::to_string_pretty(&serde_json::json!({
            "data": data,
            "errors": errors
        }))
        .unwrap_or("<error>".into());
        write!(f, "{}", resp)
    }
}

impl From<Result<Data, GraphqlError>> for Response {
    fn from(result: Result<Data, GraphqlError>) -> Self {
        match result {
            Ok(data) => Response::data(data),
            Err(err) => Response::error(err),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Data {
    Json(Bytes),
    Cbor(Bytes),
}

impl Data {
    pub fn is_json(&self) -> bool {
        matches!(self, Data::Json(_))
    }

    pub fn is_cbor(&self) -> bool {
        matches!(self, Data::Cbor(_))
    }

    pub fn as_json(&self) -> Option<&Bytes> {
        match self {
            Data::Json(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_cbor(&self) -> Option<&Bytes> {
        match self {
            Data::Cbor(bytes) => Some(bytes),
            _ => None,
        }
    }
}
