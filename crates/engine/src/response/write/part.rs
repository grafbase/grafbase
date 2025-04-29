use std::{cell::RefCell, rc::Rc, sync::Arc};

use schema::Schema;

use crate::{
    prepare::{ConcreteShapeId, PreparedOperation, ResponseObjectSetDefinitionId},
    response::{
        DataPart, ErrorPartBuilder, GraphqlError, ParentObjectId, ParentObjects, ResponseObjectField,
        ResponseObjectRef, ResponseObjectSet, ResponseValueId,
    },
};

use super::deserialize::{EntitiesSeed, EntitySeed};

pub(crate) struct ResponsePartBuilder<'ctx> {
    pub(super) schema: &'ctx Schema,
    pub(super) operation: &'ctx PreparedOperation,
    pub data: DataPart,
    pub errors: ErrorPartBuilder<'ctx>,
    pub parent_objects: Arc<ParentObjects>,
    pub(super) propagated_null_up_to_root: bool,
    pub(super) propagated_null_up_to_paths: Vec<Vec<ResponseValueId>>,
    pub(super) subgraph_errors: Vec<GraphqlError>,
    pub(super) updates: Vec<ObjectUpdate>,
    pub(super) object_sets: Vec<(ResponseObjectSetDefinitionId, ResponseObjectSet)>,
}

impl<'ctx> ResponsePartBuilder<'ctx> {
    pub(super) fn new(
        schema: &'ctx Schema,
        operation: &'ctx PreparedOperation,
        data: DataPart,
        parent_objects: Arc<ParentObjects>,
    ) -> Self {
        let errors = ErrorPartBuilder::new(operation);
        Self {
            schema,
            operation,
            data,
            errors,
            updates: vec![ObjectUpdate::Missing; parent_objects.len()],
            parent_objects,
            propagated_null_up_to_root: false,
            propagated_null_up_to_paths: Vec::new(),
            subgraph_errors: Vec::new(),
            object_sets: Vec::new(),
        }
    }

    /// Executors manipulate the response within a Send future, so we can't use a Rc/RefCell
    /// directly. Only once the executor is ready to write should it use this method.
    pub fn into_shared(self) -> SharedResponsePartBuilder<'ctx> {
        SharedResponsePartBuilder(Rc::new(RefCell::new(self)))
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

    pub fn insert_update(&mut self, id: ParentObjectId, update: ObjectUpdate) {
        self.updates[usize::from(id)] = update;
    }

    pub fn insert_errors(&mut self, error: impl Into<GraphqlError>, ids: impl IntoIterator<Item = ParentObjectId>) {
        let error: GraphqlError = error.into();
        for id in ids {
            self.insert_update(id, ObjectUpdate::Error(error.clone()));
        }
    }

    pub fn push_object_ref(&mut self, set_id: ResponseObjectSetDefinitionId, obj: ResponseObjectRef) {
        if let Some((_, set)) = self.object_sets.iter_mut().find(|(id, _)| set_id == *id) {
            set.push(obj);
        } else {
            self.object_sets.push((set_id, vec![obj]));
        }
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
pub(crate) struct SharedResponsePartBuilder<'ctx>(Rc<RefCell<ResponsePartBuilder<'ctx>>>);

impl<'ctx> std::ops::Deref for SharedResponsePartBuilder<'ctx> {
    type Target = Rc<RefCell<ResponsePartBuilder<'ctx>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SharedResponsePartBuilder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'ctx> SharedResponsePartBuilder<'ctx> {
    pub fn unshare(self) -> Option<ResponsePartBuilder<'ctx>> {
        Rc::try_unwrap(self.0).map(|part| part.into_inner()).ok()
    }

    pub fn seed(&self, shape_id: ConcreteShapeId, id: ParentObjectId) -> EntitySeed<'ctx> {
        EntitySeed::new(self.clone(), shape_id, id)
    }

    pub fn batch_seed(&self, shape_id: ConcreteShapeId) -> EntitiesSeed<'ctx> {
        EntitiesSeed::new(self.clone(), shape_id)
    }
}

#[derive(Clone)]
pub(crate) enum ObjectUpdate {
    Missing,
    Fields(Vec<ResponseObjectField>),
    Error(GraphqlError),
    PropagateNullWithoutError,
}
