use std::sync::Arc;

use async_lock::RwLock;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use grafbase_engine_parser::{types::SelectionSet, Positioned};
use graph_entities::QueryResponse;
use ulid::Ulid;

use crate::{
    registry::{resolvers::ResolvedValue, NamedType},
    ContextSelectionSet, Error, PathSegment, QueryEnv, QueryPathSegment, ResolverChainNode, SchemaEnv,
};

pub struct DeferredWorkload {
    pub label: Option<String>,
    selection_set: Positioned<SelectionSet>,
    pub path: Vec<PathSegment>,
    current_type_name: NamedType<'static>,
    pub parent_resolver_value: Option<ResolvedValue>,
}

impl DeferredWorkload {
    pub fn new(
        selection_set: Positioned<SelectionSet>,
        path: Vec<PathSegment>,
        current_type_name: NamedType<'static>,
        parent_resolver_value: Option<ResolvedValue>,
    ) -> Self {
        DeferredWorkload {
            label: None, // Will work on adding labels later
            selection_set,
            path,
            current_type_name,
            parent_resolver_value,
        }
    }

    pub fn to_context<'a>(
        &'a self,
        schema_env: &'a SchemaEnv,
        query_env: &'a QueryEnv,
        deferred_workloads: DeferredWorkloadSender,
    ) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            // Ok, all this stuff is a massive PITA
            path_node: None, // TODO: This needs to be set for errors to work properly...
            resolver_node: Some(ResolverChainNode {
                parent: None, // This will break anyone looking too far up the chain, but I don't think we care.
                segment: self
                    .path
                    .last()
                    .expect("there to always be a path")
                    .to_query_path_segment(),
                field: None,
                executable_field: None,
                ty: Some(
                    schema_env
                        .registry
                        .lookup(&self.current_type_name)
                        .expect("TODO: handle errors"),
                ),
                selections: Some(&self.selection_set),
                execution_id: Ulid::new(),
                resolver: None,
            }),
            item: &self.selection_set,
            schema_env,
            query_env,
            resolvers_data: Default::default(),
            response_graph: Arc::new(RwLock::new(QueryResponse::default())),
            deferred_workloads: Some(deferred_workloads),
        }
    }
}

impl PathSegment {
    pub fn to_query_path_segment(&self) -> QueryPathSegment<'_> {
        match self {
            PathSegment::Field(name) => QueryPathSegment::Name(name.as_str()),
            PathSegment::Index(index) => QueryPathSegment::Index(*index),
        }
    }
}

#[derive(Clone)]
pub struct DeferredWorkloadSender(UnboundedSender<DeferredWorkload>);

impl DeferredWorkloadSender {
    pub fn send(&self, workload: DeferredWorkload) -> Result<(), Error> {
        self.0
            .unbounded_send(workload)
            .map_err(|error| Error::new(error.to_string()))
    }
}

pub struct DeferredWorkloadReceiver(UnboundedReceiver<DeferredWorkload>);

impl DeferredWorkloadReceiver {
    pub fn receive(&mut self) -> Option<DeferredWorkload> {
        self.0.try_next().ok().flatten()
    }
}

pub fn workload_channel() -> (DeferredWorkloadSender, DeferredWorkloadReceiver) {
    let (sender, receiver) = futures::channel::mpsc::unbounded();
    (DeferredWorkloadSender(sender), DeferredWorkloadReceiver(receiver))
}
