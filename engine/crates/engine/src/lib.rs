//! # A GraphQL server library implemented in Rust
#![deny(clippy::all)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::match_wildcard_for_single_variants)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(unused_crate_dependencies)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::future_not_send)]
#![allow(clippy::if_not_else)]
#![allow(clippy::iter_without_into_iter)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::map_flatten)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::panic)]
#![allow(clippy::redundant_else)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unused_self)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::use_self)]
#![allow(clippy::useless_let_if_seq)]
#![allow(elided_lifetimes_in_paths)]
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(target_arch = "wasm")]
use getrandom as _;

mod base;
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
    Error, ErrorExtensionValues, ErrorExtensions, InputValueError, InputValueResult, ParseRequestError, Result,
    ResultExt, ServerError, ServerResult,
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
pub use request::{BatchRequest, Request};
#[doc(no_inline)]
pub use resolver_utils::{ContainerType, LegacyEnumType, LegacyScalarType};
pub use response::{BatchResponse, GraphQlResponse, IncrementalPayload, InitialResponse, Response, StreamingPayload};
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
