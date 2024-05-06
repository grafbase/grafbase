//! Query context.

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Formatter},
    ops::Deref,
    sync::{Arc, Mutex},
};

use async_lock::Mutex as AsyncMutex;
use engine_parser::types::OperationType;
use engine_value::{ConstValue as Value, Variables};
use fnv::FnvHashMap;
use futures::{future::BoxFuture, stream::FuturesUnordered, StreamExt};
use graph_entities::QueryResponse;
use http::header::HeaderMap;

pub use self::selection_set::ContextSelectionSet;
pub(crate) use self::{
    ext::{Context, ContextExt},
    field::ContextField,
    legacy::ContextSelectionSetLegacy,
    list::{ContextList, ContextWithIndex},
};
use crate::{
    current_datetime::CurrentDateTime,
    deferred::DeferredWorkloadSender,
    extensions::Extensions,
    parser::types::{Field, FragmentDefinition, OperationDefinition, Selection, SelectionSet},
    query_path::QueryPath,
    registry::type_kinds::SelectionSetTarget,
    request::IntrospectionState,
    schema::SchemaEnv,
    CacheInvalidation, Name, Positioned, Result, ServerError, ServerResult, UploadValue,
};
pub use ext::TraceId;

mod ext;
mod field;
mod legacy;
mod list;
mod selection_set;

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
    pub response_http_headers: Mutex<HeaderMap>,
    pub introspection_state: IntrospectionState,
    pub errors: Mutex<Vec<ServerError>>,
    /// Defines the current timestamp to be used whenever Utc::now() is used to have consistent
    /// datetimes (createdAt/updatedAt typically) across objects
    pub current_datetime: CurrentDateTime,
    pub cache_invalidations: HashSet<CacheInvalidation>,
    pub response: AsyncMutex<QueryResponse>,
    /// A sender for deferred workloads (used by @defer & @stream)
    ///
    /// This is set to `None` when the user uses a transport that doesn't support
    /// incremental delivery.  In these circumstances we should not defer any workloads
    /// and just return the data as part of the main response.
    pub deferred_workloads: Option<DeferredWorkloadSender>,
    pub futures_spawner: QueryFutureSpawner,
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
    pub fn create_context<'a>(
        &'a self,
        schema_env: &'a SchemaEnv,
        item: &'a Positioned<SelectionSet>,
        root_type: SelectionSetTarget<'a>,
    ) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            ty: root_type,
            path: QueryPath::empty(),
            item,
            schema_env,
            query_env: self,
        }
    }
}

pub struct QueryEnvBuilder(QueryEnvInner);

impl QueryEnvBuilder {
    pub fn new(inner: QueryEnvInner) -> Self {
        Self(inner)
    }

    pub fn operation_type(&self) -> OperationType {
        self.0.operation.node.ty
    }

    pub fn with_deferred_sender(mut self, sender: DeferredWorkloadSender) -> Self {
        self.0.deferred_workloads = Some(sender);
        self
    }

    pub fn build(self) -> QueryEnv {
        QueryEnv::new(self.0)
    }
}

#[cfg(nope)]
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

pub enum QueryByVariables {
    ID(String),
    Constraint { key: String, value: Value },
}

/// Selection field.
#[derive(Clone, Copy)]
pub struct SelectionField<'a> {
    pub(crate) fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    pub(crate) field: &'a Field,
    pub(crate) context: &'a ContextField<'a>,
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

    /// Gets an argument with the given name. Please do _not_ use this with complex objects,
    /// only simple values (or variables). We will be burning all this code in a big fire
    /// when we implement connectors in engine-v2 anyhow...
    pub fn get_argument(&self, name: &str) -> Option<Value> {
        match self.field.get_argument(name) {
            Some(value) => value
                .node
                .clone()
                .into_const_with(|name| self.context.var_value(&name, value.pos))
                .ok(),
            None => None,
        }
    }

    /// True, if selecting nested fields from the given object.
    pub fn has_nested_items(&self) -> bool {
        !self.field.selection_set.node.items.is_empty()
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
            .field("selection_set", &DebugSelectionSet(self.selection_set().collect()))
            .finish()
    }
}

struct SelectionFieldsIter<'a> {
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    iter: Vec<std::slice::Iter<'a, Positioned<Selection>>>,
    context: &'a ContextField<'a>,
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
                        if let Some(fragment) = self.fragments.get(&fragment_spread.node.fragment_name.node) {
                            self.iter.push(fragment.node.selection_set.node.items.iter());
                        }
                    }
                    Selection::InlineFragment(inline_fragment) => {
                        self.iter.push(inline_fragment.node.selection_set.node.items.iter());
                    }
                },
                None => {
                    self.iter.pop();
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct QueryFutureSpawner {
    sender: futures::channel::mpsc::UnboundedSender<BoxFuture<'static, ()>>,
}

pub struct QuerySpawnedFuturesWaiter {
    receiver: futures::channel::mpsc::UnboundedReceiver<BoxFuture<'static, ()>>,
}

pub fn new_futures_spawner() -> (QueryFutureSpawner, QuerySpawnedFuturesWaiter) {
    let (sender, receiver) = futures_channel::mpsc::unbounded();
    (QueryFutureSpawner { sender }, QuerySpawnedFuturesWaiter { receiver })
}

impl QueryFutureSpawner {
    pub fn spawn(&self, future: BoxFuture<'static, ()>) {
        self.sender.unbounded_send(future).unwrap();
    }
}

impl QuerySpawnedFuturesWaiter {
    pub async fn wait_until_no_spawners_left(mut self) {
        let mut pool = FuturesUnordered::new();
        loop {
            futures_util::select! {
                _ = pool.next() => {},
                future = self.receiver.next() => {
                    if let Some(future) = future {
                        pool.push(future);
                    } else {
                        return;
                    }
                }
            }
        }
    }
}
