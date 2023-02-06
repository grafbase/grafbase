//! Query context.

use async_lock::RwLock as AsynRwLock;

use dynamodb::CurrentDateTime;
use graph_entities::QueryResponse;
use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use ulid::Ulid;

use cached::UnboundCache;
use derivative::Derivative;
use dynaql_value::{Value as InputValue, Variables};
use fnv::FnvHashMap;
use http::header::{AsHeaderName, HeaderMap, IntoHeaderName};
use http::HeaderValue;
use serde::ser::{SerializeSeq, Serializer};
use serde::Serialize;

use crate::extensions::Extensions;
use crate::parser::types::{
    Directive, Field, FragmentDefinition, OperationDefinition, Selection, SelectionSet,
};
use crate::registry::relations::MetaRelation;
use crate::registry::resolver_chain::ResolverChainNode;
use crate::registry::resolvers::{ResolvedValue, Resolver};
use crate::registry::{get_basic_type, MetaInputValue, MetaType};
use crate::registry::{Registry, SchemaID};
use crate::resolver_utils::resolve_input;
use crate::schema::SchemaEnv;
use crate::{
    Error, InputType, Lookahead, Name, PathSegment, Pos, Positioned, Result, ServerError,
    ServerResult, UploadValue, Value,
};

use arrow_schema::Schema as ArrowSchema;

#[cfg(feature = "query-planning")]
use crate::registry::variables::VariableResolveDefinition;
#[cfg(feature = "query-planning")]
use query_planning::logical_plan::LogicalPlan;
#[cfg(feature = "query-planning")]
use query_planning::logical_query::SelectionPlan;

/// Data related functions of the context.
pub trait DataContext<'a> {
    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `Error` if the specified type data does not exist.
    fn data<D: Any + Send + Sync>(&self) -> Result<&'a D>;

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// # Panics
    ///
    /// It will panic if the specified data type does not exist.
    fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D;

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D>;
}

/// Schema/Context data.
///
/// This is a type map, allowing you to store anything inside it.
#[derive(Default)]
pub struct Data(FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl Deref for Data {
    type Target = FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Data {
    /// Insert data.
    pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
        self.0.insert(TypeId::of::<D>(), Box::new(data));
    }

    #[allow(dead_code)]
    pub(crate) fn merge(&mut self, other: Data) {
        self.0.extend(other.0);
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Data").finish()
    }
}

/// Context for `SelectionSet`
pub type ContextSelectionSet<'a> = ContextBase<'a, &'a Positioned<SelectionSet>>;

/// When inside a Connection, we get the subfields asked by alias which are a relation
/// (response_key, relation)
pub fn relations_edges<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
) -> HashMap<String, &'a MetaRelation> {
    let mut result = HashMap::new();
    for selection in &ctx.item.node.items {
        match &selection.node {
            Selection::Field(field) => {
                let ctx_field = ctx.with_field(field, Some(root), Some(&ctx.item.node));
                // We do take the name and not the alias
                let field_name = ctx_field.item.node.name.node.as_str();
                let field_response_key = ctx_field.item.node.response_key().node.as_str();
                if let Some(relation) = root
                    .field_by_name(field_name)
                    .and_then(|x| x.relation.as_ref())
                {
                    result.insert(field_response_key.to_string(), relation);
                }
            }
            selection => {
                let (type_condition, selection_set) = match selection {
                    Selection::Field(_) => unreachable!(),
                    Selection::FragmentSpread(spread) => {
                        let fragment = ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                        let fragment = match fragment {
                            Some(fragment) => fragment,
                            None => {
                                // Unknown fragment
                                return HashMap::new();
                            }
                        };
                        (
                            Some(&fragment.node.type_condition),
                            &fragment.node.selection_set,
                        )
                    }
                    Selection::InlineFragment(fragment) => (
                        fragment.node.type_condition.as_ref(),
                        &fragment.node.selection_set,
                    ),
                };
                let type_condition =
                    type_condition.map(|condition| condition.node.on.node.as_str());

                let introspection_type_name = ctx.registry().introspection_type_name(root);

                let applies_concrete_object = type_condition.map_or(false, |condition| {
                    introspection_type_name == condition
                        || ctx
                            .registry()
                            .implements
                            .get(introspection_type_name)
                            .map_or(false, |interfaces| interfaces.contains(condition))
                });
                if applies_concrete_object {
                    let tailed = relations_edges(&ctx.with_selection_set(selection_set), root);
                    result.extend(tailed);
                } else if type_condition.map_or(true, |condition| root.name() == condition) {
                    // The fragment applies to an interface type.
                    todo!()
                }
            }
        }
    }
    result
}

/// Context object for resolve field
pub type Context<'a> = ContextBase<'a, &'a Positioned<Field>>;

/// Context object for execute directive.
pub type ContextDirective<'a> = ContextBase<'a, &'a Positioned<Directive>>;

/// A segment in the path to the current query.
///
/// This is a borrowed form of [`PathSegment`](enum.PathSegment.html) used during execution instead
/// of passed back when errors occur.
#[derive(Debug, Clone, Copy, Serialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum QueryPathSegment<'a> {
    /// We are currently resolving an element in a list.
    Index(usize),
    /// We are currently resolving a field in an object.
    Name(&'a str),
}

/// A path to the current query.
///
/// The path is stored as a kind of reverse linked list.
#[derive(Debug, Clone, Copy)]
pub struct QueryPathNode<'a> {
    /// The parent node to this, if there is one.
    pub parent: Option<&'a QueryPathNode<'a>>,

    /// The current path segment being resolved.
    pub segment: QueryPathSegment<'a>,
}

impl<'a> serde::Serialize for QueryPathNode<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        self.try_for_each(|segment| seq.serialize_element(segment))?;
        seq.end()
    }
}

impl<'a> Display for QueryPathNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        self.try_for_each(|segment| {
            if !first {
                write!(f, ".")?;
            }
            first = false;

            match segment {
                QueryPathSegment::Index(idx) => write!(f, "{}", *idx),
                QueryPathSegment::Name(name) => write!(f, "{name}"),
            }
        })
    }
}

impl<'a> QueryPathNode<'a> {
    /// Convert the path to a JSON Pointer
    pub fn to_json_pointer(&self) -> String {
        let mut result = String::new();
        let mut first = true;
        self.try_for_each(|segment| {
            if !first {
                write!(&mut result, "/").expect("Shouldn't fail");
            }
            first = false;

            match segment {
                QueryPathSegment::Index(idx) => write!(&mut result, "{}", *idx),
                QueryPathSegment::Name(name) => write!(&mut result, "{name}"),
            }
        })
        .expect("Shouldn't fail");
        result
    }

    /// Get the current field name.
    ///
    /// This traverses all the parents of the node until it finds one that is a field name.
    pub fn field_name(&self) -> &str {
        std::iter::once(self)
            .chain(self.parents())
            .find_map(|node| match node.segment {
                QueryPathSegment::Name(name) => Some(name),
                QueryPathSegment::Index(_) => None,
            })
            .unwrap()
    }

    /// Get the path represented by `Vec<String>`; numbers will be stringified.
    #[must_use]
    pub fn to_string_vec(self) -> Vec<String> {
        let mut res = Vec::new();
        self.for_each(|s| {
            res.push(match s {
                QueryPathSegment::Name(name) => (*name).to_string(),
                QueryPathSegment::Index(idx) => idx.to_string(),
            });
        });
        res
    }

    /// Iterate over the parents of the node.
    pub fn parents(&self) -> Parents<'_> {
        Parents(self)
    }

    pub(crate) fn for_each<F: FnMut(&QueryPathSegment<'a>)>(&self, mut f: F) {
        let _ = self.try_for_each::<std::convert::Infallible, _>(|segment| {
            f(segment);
            Ok(())
        });
    }

    pub(crate) fn try_for_each<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
        &self,
        mut f: F,
    ) -> Result<(), E> {
        self.try_for_each_ref(&mut f)
    }

    fn try_for_each_ref<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
        &self,
        f: &mut F,
    ) -> Result<(), E> {
        if let Some(parent) = &self.parent {
            parent.try_for_each_ref(f)?;
        }
        f(&self.segment)
    }
}

/// An iterator over the parents of a [`QueryPathNode`](struct.QueryPathNode.html).
#[derive(Debug, Clone)]
pub struct Parents<'a>(&'a QueryPathNode<'a>);

impl<'a> Parents<'a> {
    /// Get the current query path node, which the next call to `next` will get the parents of.
    #[must_use]
    pub fn current(&self) -> &'a QueryPathNode<'a> {
        self.0
    }
}

impl<'a> Iterator for Parents<'a> {
    type Item = &'a QueryPathNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.0.parent;
        if let Some(parent) = parent {
            self.0 = parent;
        }
        parent
    }
}

impl<'a> std::iter::FusedIterator for Parents<'a> {}

type ResolverCacheType = Arc<AsynRwLock<UnboundCache<u64, Result<ResolvedValue, Error>>>>;

/// Query context.
///
/// **This type is not stable and should not be used directly.**
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct ContextBase<'a, T> {
    /// The current path node being resolved.
    pub path_node: Option<QueryPathNode<'a>>,
    /// The current resolver path being resolved.
    pub resolver_node: Option<ResolverChainNode<'a>>,
    #[doc(hidden)]
    pub item: T,
    #[doc(hidden)]
    #[derivative(Debug = "ignore")]
    pub schema_env: &'a SchemaEnv,
    #[doc(hidden)]
    #[derivative(Debug = "ignore")]
    pub query_env: &'a QueryEnv,
    #[doc(hidden)]
    #[derivative(Debug = "ignore")]
    pub resolvers_cache: ResolverCacheType,
    #[doc(hidden)]
    /// Every Resolvers are able to store a Value inside this cache
    pub resolvers_data: Arc<RwLock<FnvHashMap<String, Box<dyn Any + Sync + Send>>>>,
    #[doc(hidden)]
    pub response_graph: Arc<AsynRwLock<QueryResponse>>,
}

#[doc(hidden)]
pub struct QueryEnvInner {
    pub extensions: Extensions,
    pub variables: Variables,
    pub operation_name: Option<String>,
    pub operation: Positioned<OperationDefinition>,
    pub fragments: HashMap<Name, Positioned<FragmentDefinition>>,
    pub uploads: Vec<UploadValue>,
    pub session_data: Arc<Data>,
    pub ctx_data: Arc<Data>,
    pub http_headers: Mutex<HeaderMap>,
    pub disable_introspection: bool,
    pub errors: Mutex<Vec<ServerError>>,
    /// Defines the current timestamp to be used whenever Utc::now() is used to have consistent
    /// datetimes (createdAt/updatedAt typically) across objects
    pub current_datetime: CurrentDateTime,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct QueryEnv(Arc<QueryEnvInner>);

impl Deref for QueryEnv {
    type Target = QueryEnvInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl QueryEnv {
    #[doc(hidden)]
    pub fn new(inner: QueryEnvInner) -> QueryEnv {
        QueryEnv(Arc::new(inner))
    }

    #[doc(hidden)]
    pub fn create_context<'a, T>(
        &'a self,
        schema_env: &'a SchemaEnv,
        path_node: Option<QueryPathNode<'a>>,
        resolver_node: Option<ResolverChainNode<'a>>,
        item: T,
    ) -> ContextBase<'a, T> {
        ContextBase {
            path_node,
            resolver_node,
            item,
            schema_env,
            query_env: self,
            resolvers_cache: Arc::new(AsynRwLock::new(UnboundCache::with_capacity(32))),
            resolvers_data: Default::default(),
            response_graph: Arc::new(AsynRwLock::new(QueryResponse::default())),
        }
    }
}

/// We suppose each time it's a [`crate::Value`] but it could happens it's not.
/// See specialized behavior of internal cache on documentation.
pub fn resolver_data_get_opt_ref<'a, D: Any + Send + Sync>(
    store: &'a FnvHashMap<String, Box<dyn Any + Sync + Send>>,
    key: &'a str,
) -> Option<&'a D> {
    store.get(key).and_then(|d| d.downcast_ref::<D>())
}

impl<'a, T> ContextBase<'a, T> {
    /// Only insert a value if a value wasn't there before.
    pub fn resolver_data_insert<D: Any + Send + Sync>(&'a self, key: String, data: D) {
        match self.resolvers_data.write().expect("to handle").entry(key) {
            Entry::Vacant(vac) => {
                vac.insert(Box::new(data));
            }
            Entry::Occupied(_) => {}
        }
    }
}

impl<'a, T> DataContext<'a> for ContextBase<'a, T> {
    fn data<D: Any + Send + Sync>(&self) -> Result<&'a D> {
        ContextBase::data::<D>(self)
    }

    fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D {
        ContextBase::data_unchecked::<D>(self)
    }

    fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
        ContextBase::data_opt::<D>(self)
    }
}

impl<'a, T> ContextBase<'a, T> {
    /// We add a new field with the Context with the proper execution_id generated.
    pub fn with_field(
        &'a self,
        field: &'a Positioned<Field>,
        ty: Option<&'a MetaType>,
        selections: Option<&'a SelectionSet>,
    ) -> ContextBase<'a, &'a Positioned<Field>> {
        let registry = &self.schema_env.registry;

        let meta_field = ty.and_then(|ty| ty.field_by_name(&field.node.name.node));

        let meta = meta_field
            .map(|x| get_basic_type(x.ty.as_str()))
            .and_then(|x| registry.types.get(x));

        ContextBase {
            path_node: Some(QueryPathNode {
                parent: self.path_node.as_ref(),
                segment: QueryPathSegment::Name(&field.node.response_key().node),
            }),
            resolver_node: Some(ResolverChainNode {
                parent: self.resolver_node.as_ref(),
                segment: QueryPathSegment::Name(&field.node.response_key().node),
                ty: meta,
                field: meta_field,
                executable_field: Some(field),
                resolver: meta_field.and_then(|x| x.resolve.as_ref()),
                transformer: meta_field.and_then(|x| x.transformer.as_ref()),
                execution_id: Ulid::from_datetime(self.query_env.current_datetime.clone().into()),
                selections,
                variables: {
                    meta_field.map(|x| {
                        x.args
                            .values()
                            .map(|y| (x.name.as_ref(), y))
                            .collect::<Vec<(&str, &MetaInputValue)>>()
                    })
                },
            }),
            item: field,
            schema_env: self.schema_env,
            query_env: self.query_env,
            resolvers_cache: self.resolvers_cache.clone(),
            resolvers_data: self.resolvers_data.clone(),
            response_graph: self.response_graph.clone(),
        }
    }

    #[doc(hidden)]
    pub fn with_selection_set(
        &self,
        selection_set: &'a Positioned<SelectionSet>,
    ) -> ContextBase<'a, &'a Positioned<SelectionSet>> {
        ContextBase {
            path_node: self.path_node,
            resolver_node: self.resolver_node.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
            resolvers_cache: self.resolvers_cache.clone(),
            resolvers_data: self.resolvers_data.clone(),
            response_graph: self.response_graph.clone(),
        }
    }

    #[doc(hidden)]
    pub fn set_error_path(&self, error: ServerError) -> ServerError {
        if let Some(node) = self.path_node {
            let mut path = Vec::new();
            node.for_each(|current_node| {
                path.push(match current_node {
                    QueryPathSegment::Name(name) => PathSegment::Field((*name).to_string()),
                    QueryPathSegment::Index(idx) => PathSegment::Index(*idx),
                })
            });
            ServerError { path, ..error }
        } else {
            error
        }
    }

    /// Report a resolver error.
    ///
    /// When implementing `OutputType`, if an error occurs, call this function to report this error and return `Value::Null`.
    pub fn add_error(&self, error: ServerError) {
        self.query_env.errors.lock().unwrap().push(error);
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `Error` if the specified type data does not exist.
    pub fn data<D: Any + Send + Sync>(&self) -> Result<&'a D> {
        self.data_opt::<D>().ok_or_else(|| {
            Error::new(format!(
                "Data `{}` does not exist.",
                std::any::type_name::<D>()
            ))
        })
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// # Panics
    ///
    /// It will panic if the specified data type does not exist.
    pub fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D {
        self.data_opt::<D>()
            .unwrap_or_else(|| panic!("Data `{}` does not exist.", std::any::type_name::<D>()))
    }

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
        self.query_env
            .ctx_data
            .0
            .get(&TypeId::of::<D>())
            .or_else(|| self.query_env.session_data.0.get(&TypeId::of::<D>()))
            .or_else(|| self.schema_env.data.0.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }

    /// Returns whether the HTTP header `key` is currently set on the response
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dynaql::*;
    /// use ::http::header::ACCESS_CONTROL_ALLOW_ORIGIN;
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn greet(&self, ctx: &Context<'_>) -> String {
    ///
    ///         let header_exists = ctx.http_header_contains("Access-Control-Allow-Origin");
    ///         assert!(!header_exists);
    ///
    ///         ctx.insert_http_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    ///
    ///         let header_exists = ctx.http_header_contains("Access-Control-Allow-Origin");
    ///         assert!(header_exists);
    ///
    ///         String::from("Hello world")
    ///     }
    /// }
    /// ```
    pub fn http_header_contains(&self, key: impl AsHeaderName) -> bool {
        self.query_env
            .http_headers
            .lock()
            .unwrap()
            .contains_key(key)
    }

    /// Sets a HTTP header to response.
    ///
    /// If the header was not currently set on the response, then `None` is returned.
    ///
    /// If the response already contained this header then the new value is associated with this key
    /// and __all the previous values are removed__, however only a the first previous
    /// value is returned.
    ///
    /// See [`http::HeaderMap`] for more details on the underlying implementation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dynaql::*;
    /// use ::http::header::ACCESS_CONTROL_ALLOW_ORIGIN;
    /// use ::http::HeaderValue;
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn greet(&self, ctx: &Context<'_>) -> String {
    ///
    ///         // Headers can be inserted using the `http` constants
    ///         let was_in_headers = ctx.insert_http_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    ///         assert_eq!(was_in_headers, None);
    ///
    ///         // They can also be inserted using &str
    ///         let was_in_headers = ctx.insert_http_header("Custom-Header", "1234");
    ///         assert_eq!(was_in_headers, None);
    ///
    ///         // If multiple headers with the same key are `inserted` then the most recent
    ///         // one overwrites the previous. If you want multiple headers for the same key, use
    ///         // `append_http_header` for subsequent headers
    ///         let was_in_headers = ctx.insert_http_header("Custom-Header", "Hello World");
    ///         assert_eq!(was_in_headers, Some(HeaderValue::from_static("1234")));
    ///
    ///         String::from("Hello world")
    ///     }
    /// }
    /// ```
    pub fn insert_http_header(
        &self,
        name: impl IntoHeaderName,
        value: impl TryInto<HeaderValue>,
    ) -> Option<HeaderValue> {
        if let Ok(value) = value.try_into() {
            self.query_env
                .http_headers
                .lock()
                .unwrap()
                .insert(name, value)
        } else {
            None
        }
    }

    /// Sets a HTTP header to response.
    ///
    /// If the header was not currently set on the response, then `false` is returned.
    ///
    /// If the response did have this header then the new value is appended to the end of the
    /// list of values currently associated with the key, however the key is not updated
    /// _(which is important for types that can be `==` without being identical)_.
    ///
    /// See [`http::HeaderMap`] for more details on the underlying implementation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dynaql::*;
    /// use ::http::header::SET_COOKIE;
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn greet(&self, ctx: &Context<'_>) -> String {
    ///         // Insert the first instance of the header
    ///         ctx.insert_http_header(SET_COOKIE, "Chocolate Chip");
    ///
    ///         // Subsequent values should be appended
    ///         let header_already_exists = ctx.append_http_header("Set-Cookie", "Macadamia");
    ///         assert!(header_already_exists);
    ///
    ///         String::from("Hello world")
    ///     }
    /// }
    /// ```
    pub fn append_http_header(
        &self,
        name: impl IntoHeaderName,
        value: impl TryInto<HeaderValue>,
    ) -> bool {
        if let Ok(value) = value.try_into() {
            self.query_env
                .http_headers
                .lock()
                .unwrap()
                .append(name, value)
        } else {
            false
        }
    }

    fn var_value(&self, name: &str, pos: Pos) -> ServerResult<Value> {
        self.query_env
            .operation
            .node
            .variable_definitions
            .iter()
            .find(|def| def.node.name.node == name)
            .and_then(|def| {
                self.query_env
                    .variables
                    .get(&def.node.name.node)
                    .or_else(|| def.node.default_value())
            })
            .cloned()
            .ok_or_else(|| ServerError::new(format!("Variable {name} is not defined."), Some(pos)))
    }

    pub fn resolve_input_value(&self, value: Positioned<InputValue>) -> ServerResult<Value> {
        let pos = value.pos;
        value
            .node
            .into_const_with(|name| self.var_value(&name, pos))
    }

    #[doc(hidden)]
    fn get_param_value<Q: InputType>(
        &self,
        arguments: &[(Positioned<Name>, Positioned<InputValue>)],
        name: &str,
        default: Option<fn() -> Q>,
    ) -> ServerResult<(Pos, Q)> {
        let value = arguments
            .iter()
            .find(|(n, _)| n.node.as_str() == name)
            .map(|(_, value)| value)
            .cloned();

        if value.is_none() {
            if let Some(default) = default {
                return Ok((Pos::default(), default()));
            }
        }
        let (pos, value) = match value {
            Some(value) => (value.pos, Some(self.resolve_input_value(value)?)),
            None => (Pos::default(), None),
        };

        InputType::parse(value)
            .map(|value| (pos, value))
            .map_err(|e| e.into_server_error(pos))
    }
}

impl<'a> ContextBase<'a, &'a Positioned<SelectionSet>> {
    #[doc(hidden)]
    #[must_use]
    pub fn with_index(
        &'a self,
        idx: usize,
        selections: Option<&'a SelectionSet>,
    ) -> ContextBase<'a, &'a Positioned<SelectionSet>> {
        ContextBase {
            path_node: Some(QueryPathNode {
                parent: self.path_node.as_ref(),
                segment: QueryPathSegment::Index(idx),
            }),
            resolver_node: Some(ResolverChainNode {
                parent: self.resolver_node.as_ref(),
                segment: QueryPathSegment::Index(idx),
                field: self.resolver_node.as_ref().and_then(|x| x.field),
                executable_field: self.resolver_node.as_ref().and_then(|x| x.executable_field),
                ty: self.resolver_node.as_ref().and_then(|x| x.ty),
                resolver: None,
                transformer: None,
                execution_id: Ulid::from_datetime(self.query_env.current_datetime.clone().into()),
                selections,
                variables: None,
            }),
            item: self.item,
            schema_env: self.schema_env,
            query_env: self.query_env,
            resolvers_cache: self.resolvers_cache.clone(),
            resolvers_data: self.resolvers_data.clone(),
            response_graph: self.response_graph.clone(),
        }
    }
}

impl<'a, T> ContextBase<'a, T> {
    /// Get the registry
    pub fn registry(&'a self) -> &'a Registry {
        &self.schema_env.registry
    }
}

impl<'a> ContextBase<'a, &'a Positioned<Field>> {
    pub fn get_schema_id(&self, id: SchemaID) -> ServerResult<Arc<ArrowSchema>> {
        self.registry().get_schema_id(id, Some(self.item.pos))
    }

    #[cfg(feature = "query-planning")]
    pub fn to_selection_plan(
        &self,
        _root: &'a MetaType,
        _previous_plan: Option<Arc<LogicalPlan>>,
    ) -> Positioned<SelectionPlan> {
        todo!()
    }

    #[cfg(feature = "query-planning")]
    /// Convert the actual field to a [`LogicalPlan`] by looking at the corresponding type on the
    /// schema.
    pub fn to_logic_plan(
        &self,
        _parent_type: &MetaType,
        previous_plan: Option<Arc<LogicalPlan>>,
    ) -> ServerResult<LogicalPlan> {
        use query_planning::logical_plan::builder::LogicalPlanBuilder;

        let resolver = self
            .resolver_node
            .as_ref()
            .and_then(|x| x.resolver)
            .map(|x| self.from_resolver_to_logic_plan(previous_plan, x));

        match resolver {
            None => Ok(LogicalPlanBuilder::empty().build()),
            Some(elt) => elt,
        }
    }

    #[cfg(feature = "query-planning")]
    /// Convert a [`VariableResolveDefinition`] into an equivalent LogicPlan
    pub fn from_variable_resolution(
        &self,
        previous_plan: Option<Arc<LogicalPlan>>,
        variable: &VariableResolveDefinition,
    ) -> ServerResult<LogicalPlan> {
        use query_planning::logical_plan::builder::LogicalPlanBuilder;
        use query_planning::reexport::arrow_schema::{DataType, Field};
        use query_planning::scalar::ScalarValue;

        let pos = self.item.pos;
        let field_name = (&self.item.node.response_key().clone().node).to_string();

        match variable {
            VariableResolveDefinition::DebugString(value) => LogicalPlanBuilder::values(
                vec![Field::new(field_name, DataType::Utf8, false)],
                vec![vec![ScalarValue::new_utf8(value)]],
            )
            .map_err(|err| ServerError::new(err.to_string(), Some(pos)))
            .map(query_planning::logical_plan::builder::LogicalPlanBuilder::build),
            VariableResolveDefinition::InputTypeName(value) => {
                // We need to have input
                let _resolved_value = self.param_value_dynamic(&value)?;
                // Convert this input to an Arrow Format
                todo!()
            },
            VariableResolveDefinition::ResolverData(_) => unreachable!("Shouldn't be used anymore"),
            VariableResolveDefinition::LocalData(value) => match previous_plan {
                Some(plan) => LogicalPlanBuilder::from(plan.as_ref().clone())
                    .projection(vec![value])
                    .map_err(|err| ServerError::new(err.to_string(), Some(pos)))
                    .map(query_planning::logical_plan::builder::LogicalPlanBuilder::build),
                None => {
                    Err(ServerError::new("You can't ask for {value} from the Input plan when no Input plan are not included.", Some(pos)))
                },
            },
        }
    }

    #[cfg(feature = "query-planning")]
    pub fn from_resolver_to_logic_plan(
        &self,
        previous_plan: Option<Arc<LogicalPlan>>,
        resolver: &Resolver,
    ) -> ServerResult<LogicalPlan> {
        use query_planning::logical_plan::builder::{join, LogicalPlanBuilder};
        use query_planning::logical_plan::Datasource;

        use crate::registry::resolvers::dynamo_querying::DynamoResolver;
        use crate::registry::resolvers::ResolverType;

        // TODO: Take the resolver from the context.

        Ok(match &resolver.r#type {
            ResolverType::DynamoResolver(DynamoResolver::QueryPKSK { pk, sk, schema }) => {
                let pk = self.from_variable_resolution(previous_plan.clone(), pk)?;
                let sk = self.from_variable_resolution(previous_plan, sk)?;
                let schema = self.get_schema_id(
                    schema.ok_or_else(|| ServerError::new("No schema id", Some(self.item.pos)))?,
                )?;

                LogicalPlanBuilder::from(
                    join(pk, sk)
                        .map_err(|err| ServerError::new(err.to_string(), Some(self.item.pos)))?,
                )
                .get_entity_by_id(&schema, Datasource::gb())
            }
            ResolverType::DynamoResolver(DynamoResolver::QueryBy { by, schema }) => {
                let _by = self.from_variable_resolution(previous_plan, by)?;
                let schema = self.get_schema_id(
                    schema.ok_or_else(|| ServerError::new("No schema id", Some(self.item.pos)))?,
                )?;

                // TODO: Add the by into values
                LogicalPlanBuilder::empty().get_entity_by_unique(&schema, Datasource::gb())
            }
            ResolverType::DynamoResolver(DynamoResolver::QuerySingleRelation {
                parent_pk: _,
                relation_name: _,
            }) => Ok(LogicalPlanBuilder::empty()),
            ResolverType::DynamoResolver(DynamoResolver::ListResultByTypePaginated { .. }) => {
                Ok(LogicalPlanBuilder::empty())
            }
            ResolverType::DynamoMutationResolver(_) => Ok(LogicalPlanBuilder::empty()),
            ResolverType::ContextDataResolver(_) => Ok(LogicalPlanBuilder::empty()),
            ResolverType::DebugResolver(_) => Ok(LogicalPlanBuilder::empty()),
        }
        .map_err(|err| ServerError::new(err.to_string(), Some(self.item.pos)))?
        .build())
    }

    #[doc(hidden)]
    pub fn param_value<T: InputType>(
        &self,
        name: &str,
        default: Option<fn() -> T>,
    ) -> ServerResult<(Pos, T)> {
        self.get_param_value(&self.item.node.arguments, name, default)
    }

    pub fn param_value_dynamic(&self, name: &str) -> ServerResult<Value> {
        let meta = self
            .resolver_node
            .as_ref()
            .and_then(|r| r.field)
            .ok_or_else(|| {
                ServerError::new(
                    "Context does not have any field associated.",
                    Some(self.item.pos),
                )
            })?;
        if let Some(meta_input_value) = meta.args.get(name) {
            let maybe_value = self
                .item
                .node
                .arguments
                .iter()
                .find(|(n, _)| n.node.as_str() == name)
                .map(|(_, value)| value)
                .cloned();

            let const_value = match maybe_value {
                Some(value) => self.resolve_input_value(value)?,
                None => Value::Null,
            };
            resolve_input(self, meta_input_value, const_value)
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
    /// use dynaql::*;
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
    /// use dynaql::*;
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
}

impl<'a> ContextBase<'a, &'a Positioned<Directive>> {
    #[doc(hidden)]
    pub fn param_value<T: InputType>(
        &self,
        name: &str,
        default: Option<fn() -> T>,
    ) -> ServerResult<(Pos, T)> {
        self.get_param_value(&self.item.node.arguments, name, default)
    }
}

/// Selection field.
#[derive(Clone, Copy)]
pub struct SelectionField<'a> {
    pub(crate) fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    pub(crate) field: &'a Field,
    pub(crate) context: &'a Context<'a>,
}

impl<'a> SelectionField<'a> {
    /// Get the name of this field.
    #[inline]
    pub fn name(&self) -> &'a str {
        self.field.name.node.as_str()
    }

    /// Get the alias of this field.
    #[inline]
    pub fn alias(&self) -> Option<&'a str> {
        self.field.alias.as_ref().map(|alias| alias.node.as_str())
    }

    /// Get the arguments of this field.
    pub fn arguments(&self) -> ServerResult<Vec<(Name, Value)>> {
        let mut arguments = Vec::with_capacity(self.field.arguments.len());
        for (name, value) in &self.field.arguments {
            let pos = name.pos;
            arguments.push((
                name.node.clone(),
                value
                    .clone()
                    .node
                    .into_const_with(|name| self.context.var_value(&name, pos))?,
            ));
        }
        Ok(arguments)
    }

    /// Get all subfields of the current selection set.
    pub fn selection_set(&self) -> impl Iterator<Item = SelectionField<'a>> {
        SelectionFieldsIter {
            fragments: self.fragments,
            iter: vec![self.field.selection_set.node.items.iter()],
            context: self.context,
        }
    }
}

impl<'a> Debug for SelectionField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        struct DebugSelectionSet<'a>(Vec<SelectionField<'a>>);

        impl<'a> Debug for DebugSelectionSet<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.debug_list().entries(&self.0).finish()
            }
        }

        f.debug_struct(self.name())
            .field("name", &self.name())
            .field(
                "selection_set",
                &DebugSelectionSet(self.selection_set().collect()),
            )
            .finish()
    }
}

struct SelectionFieldsIter<'a> {
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    iter: Vec<std::slice::Iter<'a, Positioned<Selection>>>,
    context: &'a Context<'a>,
}

impl<'a> Iterator for SelectionFieldsIter<'a> {
    type Item = SelectionField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let it = self.iter.last_mut()?;
            let item = it.next();

            match item {
                Some(selection) => match &selection.node {
                    Selection::Field(field) => {
                        return Some(SelectionField {
                            fragments: self.fragments,
                            field: &field.node,
                            context: self.context,
                        });
                    }
                    Selection::FragmentSpread(fragment_spread) => {
                        if let Some(fragment) =
                            self.fragments.get(&fragment_spread.node.fragment_name.node)
                        {
                            self.iter
                                .push(fragment.node.selection_set.node.items.iter());
                        }
                    }
                    Selection::InlineFragment(inline_fragment) => {
                        self.iter
                            .push(inline_fragment.node.selection_set.node.items.iter());
                    }
                },
                None => {
                    self.iter.pop();
                }
            }
        }
    }
}
