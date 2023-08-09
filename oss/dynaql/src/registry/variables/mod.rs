#![allow(deprecated)]

//! Variable Resolving definition strategy is explained here.
//!
//! When you need a Variable inside a Resolver, you can use a
//! `VariableResolveDefinition` struct to define how the graphql server should
//! resolve this variable.
use std::borrow::Borrow;

use dynaql_value::{ConstValue, Name};
use grafbase_runtime::search::GraphqlCursor;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};

use self::oneof::OneOf;
use crate::{
    context::resolver_data_get_opt_ref, resolver_utils::InputResolveMode, Context, Error, ServerError, ServerResult,
    Value,
};

pub mod id;
pub mod oneof;

/// Describe what should be done by the GraphQL Server to resolve this Variable.
#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum VariableResolveDefinition {
    /// A Debug VariableResolveDefinition where you can just put the Value you
    /// would like to have.
    DebugString(String),
    /// Check the last Resolver in the Query Graph and try to resolve the
    /// variable defined in this field.
    InputTypeName(String),
    /// Check the last Resolver in the Query Graph, try to resolve the
    /// variable defined in this field and then apply connector transforms
    ConnectorInputTypeName(String),
    /// Resolve a Value by querying the ResolverContextData with a key_id.
    /// What is store in the ResolverContextData is described on each Resolver
    /// implementation.
    ///
    #[deprecated = "Should not use Context anymore in SDL def"]
    ResolverData(String),
    /// Resolve a Value by querying the most recent ancestor resolver property.
    LocalData(String),
}

impl VariableResolveDefinition {
    /// Resolve the first variable with this definition
    pub fn param<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<Option<Value>, ServerError> {
        match self {
            Self::InputTypeName(name) => ctx.param_value_dynamic(name, InputResolveMode::Default),
            Self::ConnectorInputTypeName(name) => {
                ctx.param_value_dynamic(name, InputResolveMode::ApplyConnectorTransforms)
            }
            #[allow(deprecated)]
            Self::ResolverData(key) => Ok(resolver_data_get_opt_ref::<Value>(
                &ctx.resolvers_data.read().expect("handle"),
                key,
            )
            .map(std::clone::Clone::clone)),
            Self::DebugString(inner) => Ok(Some(Value::String(inner.clone()))),
            Self::LocalData(inner) => {
                let result = last_resolver_value
                    .and_then(|x| x.get(inner))
                    .map(std::borrow::ToOwned::to_owned)
                    .unwrap_or_else(|| serde_json::Value::Null);

                Ok(Value::from_json(result).ok())
            }
        }
    }

    pub fn resolve<T: DeserializeOwned>(
        &self,
        ctx: &Context<'_>,
        last_resolver_value: Option<impl Borrow<serde_json::Value>>,
    ) -> ServerResult<T> {
        let param = match last_resolver_value {
            Some(v) => self.param(ctx, Some(v.borrow())),
            None => self.param(ctx, None),
        }?
        .unwrap_or(ConstValue::Null);
        T::deserialize(param).map_err(|err| ServerError::new(err.to_string(), Some(ctx.item.pos)))
    }

    pub fn expect_string<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<String, ServerError> {
        match self.param(ctx, last_resolver_value)? {
            Some(Value::String(inner)) => Ok(inner),
            _ => Err(Error::new("Internal Error: failed to infer key").into_server_error(ctx.item.pos)),
        }
    }

    pub fn expect_obj<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<IndexMap<Name, Value>, ServerError> {
        match self.param(ctx, last_resolver_value)? {
            Some(Value::Object(inner)) => Ok(inner),
            _ => Err(Error::new("Internal Error: failed to infer key").into_server_error(ctx.item.pos)),
        }
    }

    pub fn expect_op_obj<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<Option<IndexMap<Name, Value>>, ServerError> {
        match self.param(ctx, last_resolver_value)? {
            Some(Value::Object(inner)) => Ok(Some(inner)),
            None => Ok(None),
            _ => Err(Error::new("Internal Error: failed to infer key").into_server_error(ctx.item.pos)),
        }
    }

    pub fn expect_opt_string<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<Option<String>, ServerError> {
        match self.param(ctx, last_resolver_value)? {
            Some(Value::String(inner)) => Ok(Some(inner)),
            Some(Value::Null) => Ok(None),
            None => Ok(None),
            _ => Err(Error::new("Internal Error: failed to infer key").into_server_error(ctx.item.pos)),
        }
    }

    pub fn expect_opt_int<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
        limit: Option<usize>,
    ) -> Result<Option<usize>, ServerError> {
        let result = match self.param(ctx, last_resolver_value)? {
            Some(Value::Number(inner)) => inner
                .as_u64()
                .ok_or_else(|| Error::new("Internal Error: failed to infer Int"))
                .map(|value| value as usize)
                .and_then(|value| match limit {
                    Some(limit) if value > limit => Err(Error::new(format!(
                        "Limit Error: the integer must be smaller than {limit}"
                    ))),
                    _ => Ok(Some(value)),
                }),
            Some(Value::Null) | None => Ok(None),
            _ => Err(Error::new("Internal Error: failed to infer key")),
        };
        result.map_err(|err| err.into_server_error(ctx.item.pos))
    }

    pub fn expect_opt_cursor<'a>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<Option<String>, ServerError> {
        match self.expect_opt_string(ctx, last_resolver_value)? {
            Some(s) => match GraphqlCursor::try_from(s).map(|x| String::from_utf8(x.into_bytes())) {
                Ok(Ok(cursor)) => Ok(Some(cursor)),
                Err(_) | Ok(Err(_)) => Err(Error::new("Invalid Cursor").into_server_error(ctx.item.pos)),
            },
            None => Ok(None),
        }
    }

    pub fn expect_oneof<'a, T>(
        &self,
        ctx: &'a Context<'a>,
        last_resolver_value: Option<&'a serde_json::Value>,
    ) -> Result<Option<OneOf<T>>, ServerError>
    where
        T: Serialize + DeserializeOwned,
    {
        match self.param(ctx, last_resolver_value)? {
            Some(s) => serde_json::to_value(s)
                .and_then(serde_json::from_value)
                .map_err(|err| Error::new_with_source(err).into_server_error(ctx.item.pos)),
            None => Ok(None),
        }
    }
}
