//! # A GraphQL server library implemented in Rust
#![allow(clippy::panic)]
#![allow(clippy::upper_case_acronyms)]
#![allow(elided_lifetimes_in_paths)]
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(target_arch = "wasm")]
use getrandom as _;

mod base;
mod current_datetime;
mod error;
mod guard;
mod headers;
mod look_ahead;
#[doc(hidden)]
pub mod model;
pub mod names;
mod query_path;
mod request;
mod response;
mod schema;
mod subscription;
pub mod validation;

pub mod auth;
pub mod context;
pub mod extensions;
pub mod http;

pub mod resolver_utils;
pub mod types;
#[doc(hidden)]
pub mod validators;

pub mod graph;

mod deferred;
mod directive;
mod persisted_query;
pub use persisted_query::AutomaticPersistedQuery;
pub mod registry;

#[doc(hidden)]
pub use async_stream;
#[doc(hidden)]
pub use async_trait;
pub use auth::*;
pub use base::{
    ComplexObject, Description, LegacyInputObjectType, LegacyInputType, LegacyInterfaceType, LegacyOutputType,
    LegacyUnionType, ObjectType,
};
#[doc(hidden)]
pub use context::ContextSelectionSet;
pub use context::*;
pub use engine_parser as parser;
pub use engine_value::{
    from_value, to_value, value, ConstValue as Value, DeserializerError, Name, Number, SerializerError, Variables,
};
pub use error::{
    Error, ErrorCode, ErrorExtensionValues, ErrorExtensions, InputValueError, InputValueResult, ParseRequestError,
    Result, ResultExt, ServerError, ServerResult,
};
pub use extensions::ResolveFut;
#[doc(hidden)]
pub use futures_util;
#[doc(hidden)]
pub use graph_entities;
pub use graph_entities::ResponseNodeId;
pub use guard::{Guard, GuardExt};
pub use headers::RequestHeaders;
#[doc(hidden)]
pub use indexmap;
pub use look_ahead::Lookahead;
#[doc(no_inline)]
pub use parser::{Pos, Positioned};
pub use query_path::{QueryPath, QueryPathSegment};
pub use registry::{CacheControl, CacheInvalidation, Registry};
pub use request::{BatchRequest, OperationPlanCacheKey, PersistedQueryRequestExtension, Request, RequestExtensions};
#[doc(no_inline)]
pub use resolver_utils::{ContainerType, LegacyEnumType, LegacyScalarType};
pub use response::{
    BatchResponse, GraphQlResponse, IncrementalPayload, InitialResponse, Response, ResponseOperation, StreamingPayload,
};
pub use schema::{Schema, SchemaBuilder, SchemaEnv};
#[doc(hidden)]
pub use static_assertions;
#[doc(hidden)]
pub use subscription::SubscriptionType;
pub use types::*;
pub use validation::{ValidationMode, ValidationResult, VisitorContext};
pub use validators::CustomValidator;

/// An alias of [engine::Error](struct.Error.html). Present for backward compatibility
/// reasons.
pub type FieldError = Error;

/// An alias of [engine::Result](type.Result.html). Present for backward compatibility
/// reasons.
pub type FieldResult<T> = Result<T>;

pub use engine_derive::{
    ComplexObject, Description, Enum, InputObject, Interface, MergedObject, MergedSubscription, NewType, Object,
    Scalar, SimpleObject, Union,
};
