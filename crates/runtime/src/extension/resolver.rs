use std::{future::Future, sync::Arc};

use bytes::Bytes;
use engine_schema::{ExtensionDirective, FieldDefinition};
use error::GraphqlError;
use event_queue::EventQueue;
use futures_util::stream::BoxStream;

use crate::extension::Anything;

pub trait SelectionSet<'a>: Sized + Send + 'a {
    type Field: Field<'a, SelectionSet = Self>;
    fn requires_typename(&self) -> bool;
    fn fields_ordered_by_parent_entity(&self) -> impl Iterator<Item = Self::Field>;
}

pub trait Field<'a>: Sized + Send + 'a {
    type SelectionSet: SelectionSet<'a>;
    fn alias(&self) -> Option<&'a str>;
    fn definition(&self) -> FieldDefinition<'a>;
    fn arguments(&self) -> Option<ArgumentsId>;
    fn selection_set(&self) -> Option<Self::SelectionSet>;
    // For test purposes. Don't use it for production code, it's just slower.
    fn as_dyn(&self) -> Box<dyn DynField<'a>>;
}

pub trait DynSelectionSet<'a>: Send + 'a {
    fn requires_typename(&self) -> bool;
    fn fields_ordered_by_parent_entity(&self) -> Vec<Box<dyn DynField<'a>>>;
}

pub trait DynField<'a>: Send + 'a {
    fn alias(&self) -> Option<&'a str>;
    fn definition(&self) -> FieldDefinition<'a>;
    fn arguments(&self) -> Option<ArgumentsId>;
    fn selection_set(&self) -> Option<Box<dyn DynSelectionSet<'a>>>;
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ArgumentsId(pub u16);

impl From<ArgumentsId> for u16 {
    fn from(id: ArgumentsId) -> Self {
        id.0
    }
}

pub trait ResolverExtension<OperationContext>: Send + Sync + 'static {
    fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        event_queue: Arc<EventQueue>,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send;

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        ctx: OperationContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Response> + Send + 'f
    where
        'ctx: 'f;

    fn resolve_subscription<'ctx, 'resp, 'f>(
        &'ctx self,
        ctx: OperationContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = BoxStream<'f, Response>> + Send + 'f
    where
        'ctx: 'f;
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
        write!(f, "{resp}")
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
