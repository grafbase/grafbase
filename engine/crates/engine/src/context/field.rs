use std::{
    borrow::Cow,
    collections::HashSet,
    fmt::{self, Formatter},
};

use engine_value::ConstValue;
use serde::de::DeserializeOwned;

use crate::{
    parser::types::{Field, SelectionSet},
    query_path::QueryPath,
    registry::{type_kinds::InputType, variables::VariableResolveDefinition},
    resolver_utils::{resolve_input, InputResolveMode},
    schema::SchemaEnv,
    ContextSelectionSet, LegacyInputType, Lookahead, Pos, Positioned, QueryEnv, ResolverChainNode, SelectionField,
    ServerError, ServerResult,
};

use super::ContextExt;

/// Context when we're resolving a `Field`
#[derive(Clone)]
pub struct ContextField<'a> {
    /// The current path being resolved.
    pub path: QueryPath,
    /// The current resolver path being resolved.
    pub resolver_node: Option<ResolverChainNode<'a>>,
    /// The selection set being resolved
    pub item: &'a Positioned<Field>,
    /// Context scoped to the current schema
    pub schema_env: &'a SchemaEnv,
    /// Context scoped to the current query
    pub query_env: &'a QueryEnv,
}

impl ContextExt for ContextField<'_> {
    fn path(&self) -> &QueryPath {
        &self.path
    }

    fn query_env(&self) -> &QueryEnv {
        self.query_env
    }

    fn schema_env(&self) -> &SchemaEnv {
        self.schema_env
    }
}

impl std::fmt::Debug for ContextField<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextField")
            .field("path", &self.path)
            .field("item", &self.item)
            .finish()
    }
}

impl<'a> ContextField<'a> {
    pub fn with_selection_set(&self, selection_set: &'a Positioned<SelectionSet>) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            path: self.path.clone(),
            resolver_node: self.resolver_node.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    #[doc(hidden)]
    pub fn param_value<T: LegacyInputType>(&self, name: &str, default: Option<fn() -> T>) -> ServerResult<(Pos, T)> {
        self.get_param_value(&self.item.node.arguments, name, default)
    }

    pub fn find_argument_type(&self, name: &str) -> ServerResult<InputType<'_>> {
        let meta = self
            .resolver_node
            .as_ref()
            .and_then(|r| r.field)
            .ok_or_else(|| ServerError::new("Context does not have any field associated.", Some(self.item.pos)))?;

        meta.args
            .get(name)
            .ok_or_else(|| {
                ServerError::new(
                    &format!("Internal Error: Unknown argument '{name}'"),
                    Some(self.item.pos),
                )
            })
            .and_then(|input| {
                self.schema_env
                    .registry
                    .lookup(&input.ty)
                    .map_err(|error| error.into_server_error(self.item.pos))
            })
    }

    pub fn param_value_dynamic(&self, name: &str, mode: InputResolveMode) -> ServerResult<Option<ConstValue>> {
        let meta = self
            .resolver_node
            .as_ref()
            .and_then(|r| r.field)
            .ok_or_else(|| ServerError::new("Context does not have any field associated.", Some(self.item.pos)))?;
        if let Some(meta_input_value) = meta.args.get(name) {
            let maybe_value = self
                .item
                .node
                .arguments
                .iter()
                .find(|(n, _)| n.node.as_str() == name)
                .map(|(_, value)| value)
                .cloned()
                .map(|value| self.resolve_input_value(value))
                .transpose()?;

            resolve_input(self, name, meta_input_value, maybe_value, mode)
        } else {
            Err(ServerError::new(
                &format!("Internal Error: Unknown argument '{name}'"),
                Some(self.item.pos),
            ))
        }
    }

    /// When inside a Connection, we get the subfields asked
    pub fn relations_edges(&self) -> HashSet<String> {
        if let Some(iter) = self
            .field()
            .selection_set()
            .find(|field| field.name() == "edges")
            .and_then(|field| {
                field
                    .selection_set()
                    .find(|inner_field| inner_field.name() == "node")
                    .map(|inner_field| inner_field.selection_set())
            })
        {
            iter.map(|field| field.name().to_string()).collect()
        } else {
            HashSet::new()
        }
    }

    /// Creates a uniform interface to inspect the forthcoming selections.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use engine::*;
    ///
    /// #[derive(SimpleObject)]
    /// struct Detail {
    ///     c: i32,
    ///     d: i32,
    /// }
    ///
    /// #[derive(SimpleObject)]
    /// struct MyObj {
    ///     a: i32,
    ///     b: i32,
    ///     detail: Detail,
    /// }
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn obj(&self, ctx: &Context<'_>) -> MyObj {
    ///         if ctx.look_ahead().field("a").exists() {
    ///             // This is a query like `obj { a }`
    ///         } else if ctx.look_ahead().field("detail").field("c").exists() {
    ///             // This is a query like `obj { detail { c } }`
    ///         } else {
    ///             // This query doesn't have `a`
    ///         }
    ///         unimplemented!()
    ///     }
    /// }
    /// ```
    pub fn look_ahead(&self) -> Lookahead {
        Lookahead::new(&self.query_env.fragments, &self.item.node, self)
    }

    /// Get the current field.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// use engine::*;
    ///
    /// #[derive(SimpleObject)]
    /// struct MyObj {
    ///     a: i32,
    ///     b: i32,
    ///     c: i32,
    /// }
    ///
    /// pub struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn obj(&self, ctx: &Context<'_>) -> MyObj {
    ///         let fields = ctx.field().selection_set().map(|field| field.name()).collect::<Vec<_>>();
    ///         assert_eq!(fields, vec!["a", "b", "c"]);
    ///         MyObj { a: 1, b: 2, c: 3 }
    ///     }
    /// }
    ///
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    /// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    /// assert!(schema.execute("{ obj { a b c }}").await.is_ok());
    /// assert!(schema.execute("{ obj { a ... { b c } }}").await.is_ok());
    /// assert!(schema.execute("{ obj { a ... BC }} fragment BC on MyObj { b c }").await.is_ok());
    /// # });
    ///
    /// ```
    pub fn field(&self) -> SelectionField {
        SelectionField {
            fragments: &self.query_env.fragments,
            field: &self.item.node,
            context: self,
        }
    }

    pub fn input_by_name<T: DeserializeOwned>(&self, name: impl Into<Cow<'static, str>>) -> ServerResult<T> {
        let resolve_definition = VariableResolveDefinition::input_type_name(name);
        resolve_definition.resolve(self, Option::<serde_json::Value>::None)
    }
}
