use std::{
    borrow::Cow,
    collections::HashSet,
    fmt::{self, Formatter},
};

use engine_value::ConstValue;
use serde::de::DeserializeOwned;
use ulid::Ulid;

use super::ContextExt;
use crate::{
    parser::types::{Field, SelectionSet},
    query_path::QueryPath,
    registry::{
        type_kinds::{InputType, OutputType, SelectionSetTarget},
        variables::VariableResolveDefinition,
        MetaField, NamedType,
    },
    resolver_utils::{resolve_input, InputResolveMode},
    schema::SchemaEnv,
    Context, ContextSelectionSet, LegacyInputType, Lookahead, Pos, Positioned, QueryEnv, QueryPathSegment,
    SelectionField, ServerError, ServerResult,
};

/// Context when we're resolving a `Field`
#[derive(Clone)]
pub struct ContextField<'a> {
    /// The field in the schema
    pub field: &'a MetaField,
    /// The field in the query
    pub item: &'a Positioned<Field>,
    /// The type that contains ths field
    pub parent_type: SelectionSetTarget<'a>,

    /// The execution_id for resolving this field
    /// This is currently used for caching and feeds into NodeId in some way
    /// I don't quite understand
    pub execution_id: Ulid,

    /// The current path within query
    pub path: QueryPath,
    /// Context scoped to the current schema
    pub schema_env: &'a SchemaEnv,
    /// Context scoped to the current query
    pub query_env: &'a QueryEnv,
}

impl<'a> Context<'a> for ContextField<'a> {
    fn path(&self) -> &QueryPath {
        &self.path
    }

    fn query_env(&self) -> &'a QueryEnv {
        self.query_env
    }

    fn schema_env(&self) -> &'a SchemaEnv {
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
            ty: self.field_base_type().try_into().expect("this should work"),
            path: self.path.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    /// Builds a context for resolving an `@requires` FieldSet for this contexts field.
    pub fn with_requires_selection_set(&self, selection_set: &'a Positioned<SelectionSet>) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            ty: self.parent_type,
            path: self.path.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    pub fn to_join_context(
        &self,
        item: &'a Positioned<Field>,
        field: &'a MetaField,
        parent_type: SelectionSetTarget<'a>,
    ) -> Self {
        Self {
            field,
            item,
            parent_type,
            ..self.clone()
        }
    }

    /// Returns the base type for the currently resolving field
    pub fn field_base_type(&self) -> OutputType<'a> {
        self.registry()
            .lookup(&self.field.ty)
            .expect("a field type was missing in the registry, eek")
    }

    #[doc(hidden)]
    pub fn param_value<T: LegacyInputType>(&self, name: &str, default: Option<fn() -> T>) -> ServerResult<(Pos, T)> {
        self.get_param_value(&self.item.node.arguments, name, default)
    }

    pub fn find_argument_type(&self, name: &str) -> ServerResult<InputType<'_>> {
        self.field
            .args
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
        if let Some(meta_input_value) = self.field.args.get(name) {
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

    pub fn response_path(&self) -> Option<ResponsePath<'_>> {
        use engine_parser::types::{BaseType, Type};

        let mut keys_and_typenames = vec![];
        let mut current_type =
            engine_parser::types::Type::new(self.registry().root_type(self.query_env.0.operation.ty).name())
                .expect("the root type name should not be malformed");

        for key in self.path.iter() {
            match (key, current_type.base) {
                (QueryPathSegment::Index(_), BaseType::List(inner)) => {
                    keys_and_typenames.push((key, inner.to_string()));
                    current_type = *inner;
                }
                (QueryPathSegment::Field(field_name), BaseType::Named(name)) => {
                    let output_type = &self
                        .registry()
                        .lookup_expecting::<OutputType<'_>>(&NamedType::from(&name))
                        .ok()?;

                    let field_type = output_type.field(field_name)?.ty.as_str();

                    keys_and_typenames.push((key, field_type.to_string()));

                    current_type = Type::new(field_type)?;
                }
                _ => {
                    return None;
                }
            }
        }

        keys_and_typenames.into_iter().fold(None, |prev, (key, typename)| {
            Some(ResponsePath {
                key,
                prev: prev.map(Box::new),
                typename,
            })
        })
    }
}

#[derive(serde::Serialize)]
pub struct ResponsePath<'a> {
    key: &'a QueryPathSegment,
    prev: Option<Box<ResponsePath<'a>>>,
    typename: String,
}
