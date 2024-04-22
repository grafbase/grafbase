use engine_parser::{types::SelectionSet, Positioned};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    registry::{resolvers::ResolvedValue, NamedType, RegistryV2Ext},
    ContextSelectionSet, Error, QueryEnv, QueryPath, SchemaEnv,
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
        label: Option<String>,
        selection_set: Positioned<SelectionSet>,
        path: QueryPath,
        current_type_name: NamedType<'static>,
        parent_resolver_value: Option<ResolvedValue>,
    ) -> Self {
        DeferredWorkload {
            label,
            selection_set,
            path,
            current_type_name,
            parent_resolver_value,
        }
    }

    pub fn to_context<'a>(&'a self, schema_env: &'a SchemaEnv, query_env: &'a QueryEnv) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            path: self.path.clone(),
            ty: schema_env
                .registry
                .lookup_expecting(&self.current_type_name)
                .expect("current type name to exist"),
            item: &self.selection_set,
            schema_env,
            query_env,
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
