//! # A GraphQL server library implemented in Rust
#![deny(clippy::all)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::match_wildcard_for_single_variants)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::future_not_send)]
#![allow(clippy::if_not_else)]
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

mod base;
mod custom_directive;
mod error;
mod guard;
mod look_ahead;
#[doc(hidden)]
pub mod model;
pub mod names;
mod request;
mod response;
mod schema;
mod subscription;
pub mod validation;

pub mod auth;
pub mod context;
pub mod extensions;
pub mod http;
#[cfg(feature = "query-planning")]
pub mod logical_plan_utils;

pub mod resolver_utils;
pub mod types;
#[doc(hidden)]
pub mod validators;

pub mod graph;

pub mod registry;

#[doc(hidden)]
pub use async_stream;
#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use context::ContextSelectionSet;
#[doc(hidden)]
pub use futures_util;
#[doc(hidden)]
pub use graph_entities;
#[doc(hidden)]
pub use indexmap;
#[doc(hidden)]
pub use static_assertions;
#[doc(hidden)]
pub use subscription::SubscriptionType;

pub use auth::*;
pub use base::{
    ComplexObject, Description, InputObjectType, InputType, InterfaceType, ObjectType, OutputType,
    UnionType,
};
pub use custom_directive::{CustomDirective, CustomDirectiveFactory};
pub use dynaql_parser as parser;
pub use dynaql_value::{
    from_value, to_value, value, ConstValue as Value, DeserializerError, Name, Number,
    SerializerError, Variables,
};
pub use error::{
    Error, ErrorExtensionValues, ErrorExtensions, InputValueError, InputValueResult,
    ParseRequestError, PathSegment, Result, ResultExt, ServerError, ServerResult,
};
pub use extensions::ResolveFut;
pub use graph_entities::ResponseNodeId;
pub use guard::{Guard, GuardExt};
pub use look_ahead::Lookahead;
pub use registry::{CacheControl, CacheInvalidation};
pub use request::{BatchRequest, Request};
#[doc(no_inline)]
pub use resolver_utils::{ContainerType, EnumType, ScalarType};
pub use response::GraphQlResponse;
pub use response::{BatchResponse, Response};
pub use schema::{Schema, SchemaBuilder, SchemaEnv};
pub use validation::{ValidationMode, ValidationResult, VisitorContext};
pub use validators::CustomValidator;

pub use context::*;
#[doc(no_inline)]
pub use parser::{Pos, Positioned};
pub use types::*;

/// An alias of [dynaql::Error](struct.Error.html). Present for backward compatibility
/// reasons.
pub type FieldError = Error;

/// An alias of [dynaql::Result](type.Result.html). Present for backward compatibility
/// reasons.
pub type FieldResult<T> = Result<T>;

#[doc = include_str!("docs/complex_object.md")]
pub use dynaql_derive::ComplexObject;
#[doc = include_str!("docs/description.md")]
pub use dynaql_derive::Description;
#[doc = include_str!("docs/directive.md")]
pub use dynaql_derive::Directive;
#[doc = include_str!("docs/enum.md")]
pub use dynaql_derive::Enum;
#[doc = include_str!("docs/input_object.md")]
pub use dynaql_derive::InputObject;
#[doc = include_str!("docs/interface.md")]
pub use dynaql_derive::Interface;
#[doc = include_str!("docs/merged_object.md")]
pub use dynaql_derive::MergedObject;
#[doc = include_str!("docs/merged_subscription.md")]
pub use dynaql_derive::MergedSubscription;
#[doc = include_str!("docs/newtype.md")]
pub use dynaql_derive::NewType;
#[doc = include_str!("docs/object.md")]
pub use dynaql_derive::Object;
#[cfg(feature = "unstable_oneof")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_oneof")))]
#[doc = include_str!("docs/oneof_object.md")]
pub use dynaql_derive::OneofObject;
#[doc = include_str!("docs/scalar.md")]
pub use dynaql_derive::Scalar;
#[doc = include_str!("docs/simple_object.md")]
pub use dynaql_derive::SimpleObject;
#[doc = include_str!("docs/subscription.md")]
pub use dynaql_derive::Subscription;
#[doc = include_str!("docs/union.md")]
pub use dynaql_derive::Union;
