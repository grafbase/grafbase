use std::sync::Arc;

use async_lock::RwLock;
use engine_parser::{types::SelectionSet, Positioned};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use graph_entities::QueryResponse;
use ulid::Ulid;

use crate::{
    registry::{resolvers::ResolvedValue, NamedType},
    ContextSelectionSet, Error, QueryEnv, QueryPath, ResolverChainNode, SchemaEnv,
};

#[derive(Debug)]
pub struct DeferredWorkload {
    pub label: Option<String>,
    selection_set: Positioned<SelectionSet>,
    pub path: QueryPath,
    pub current_type_name: NamedType<'static>,
    pub parent_resolver_value: Option<ResolvedValue>,
}

impl DeferredWorkload {
    pub fn new(
        selection_set: Positioned<SelectionSet>,
        path: QueryPath,
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
            path: self.path.clone(),
            resolver_node: self.path.last().cloned().map(|segment| ResolverChainNode {
                segment,
                parent: None,
                field: None,
                executable_field: None,
                ty: Some(
                    schema_env
                        .registry
                        .lookup(&self.current_type_name)
                        .expect("current type name to exist"),
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
