use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    execution::ExecutionContext,
    prepare::{ConcreteShapeId, ResponseObjectSetDefinitionId},
    response::{
        DataPart, GraphqlError, InputObjectId, InputResponseObjectSet, ResponseObjectField, ResponseObjectRef,
        ResponseObjectSet, ResponseValueId,
    },
    Runtime,
};

use super::deserialize::UpdateSeed;

pub(crate) struct SubgraphResponse {
    pub data: DataPart,
    shape_id: ConcreteShapeId,
    pub(super) input_response_object_set: Arc<InputResponseObjectSet>,
    pub(super) propagated_null_up_to_root: bool,
    pub(super) propagated_null_up_to_paths: Vec<Vec<ResponseValueId>>,
    pub(super) errors: Vec<GraphqlError>,
    pub(super) subgraph_errors: Vec<GraphqlError>,
    pub(super) updates: Vec<ObjectUpdate>,
    pub(super) response_object_sets: Vec<(ResponseObjectSetDefinitionId, ResponseObjectSet)>,
}

impl SubgraphResponse {
    pub(super) fn new(
        data: DataPart,
        shape_id: ConcreteShapeId,
        input_response_object_set: Arc<InputResponseObjectSet>,
    ) -> Self {
        Self {
            data,
            shape_id,
            updates: vec![ObjectUpdate::Missing; input_response_object_set.len()],
            input_response_object_set,
            propagated_null_up_to_root: false,
            propagated_null_up_to_paths: Vec::new(),
            errors: Vec::new(),
            subgraph_errors: Vec::new(),
            response_object_sets: Vec::new(),
        }
    }

    /// Executors manipulate the response within a Send future, so we can't use a Rc/RefCell
    /// directly. Only once the executor is ready to write should it use this method.
    pub fn as_shared_mut(&mut self) -> SubgraphResponseRefMut<'_> {
        SubgraphResponseRefMut(Rc::new(RefCell::new(self)))
    }

    pub fn propagate_null(&mut self, path: &[ResponseValueId]) {
        let Some(i) = path.iter().rev().position(|value| value.is_nullable()) else {
            self.propagated_null_up_to_root = true;
            return;
        };
        // we inverted the path.
        let i = path.len() - i - 1;

        self.propagated_null_up_to_paths.push(path[..(i + 1)].to_vec());
    }

    pub fn input_object_ref(&self, id: InputObjectId) -> &ResponseObjectRef {
        &self.input_response_object_set[id]
    }

    pub fn insert_update(&mut self, id: InputObjectId, update: ObjectUpdate) {
        self.updates[usize::from(id)] = update;
    }

    pub fn insert_errors(&mut self, error: impl Into<GraphqlError>, ids: impl IntoIterator<Item = InputObjectId>) {
        let error: GraphqlError = error.into();
        for id in ids {
            self.insert_update(id, ObjectUpdate::Error(error.clone()));
        }
    }

    pub fn push_object_ref(&mut self, set_id: ResponseObjectSetDefinitionId, obj: ResponseObjectRef) {
        if let Some((_, set)) = self.response_object_sets.iter_mut().find(|(id, _)| set_id == *id) {
            set.push(obj);
        } else {
            self.response_object_sets.push((set_id, vec![obj]));
        }
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn set_subgraph_errors(&mut self, errors: Vec<GraphqlError>) {
        self.subgraph_errors = errors;
    }
}

/// We end up writing objects or lists at various step of the de-serialization / query
/// traversal, so having a RefCell is by far the easiest. We don't need a lock as executor are
/// not expected to parallelize their work.
/// The Rc makes it possible to write errors at one place and the data in another.
#[derive(Clone)]
pub(crate) struct SubgraphResponseRefMut<'resp>(Rc<RefCell<&'resp mut SubgraphResponse>>);

impl<'resp> std::ops::Deref for SubgraphResponseRefMut<'resp> {
    type Target = Rc<RefCell<&'resp mut SubgraphResponse>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SubgraphResponseRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'resp> SubgraphResponseRefMut<'resp> {
    pub fn seed<'ctx, R: Runtime>(&self, ctx: &ExecutionContext<'ctx, R>, id: InputObjectId) -> UpdateSeed<'resp>
    where
        'ctx: 'resp,
    {
        UpdateSeed::new(*ctx, self.clone(), self.0.borrow().shape_id, id)
    }
}

#[derive(Clone)]
pub(crate) enum ObjectUpdate {
    Missing,
    Fields(Vec<ResponseObjectField>),
    Error(GraphqlError),
    PropagateNullWithoutError,
}
