use std::{cell::RefCell, rc::Rc, sync::Arc};

use schema::Schema;
use walker::Walk as _;

use crate::{
    prepare::{
        ConcreteShapeId, ObjectIdentifier, OperationPlanContext, PreparedOperation, ResponseObjectSetDefinitionId,
    },
    response::{
        DataPart, ErrorPartBuilder, GraphqlError, ParentObjectId, ParentObjects, ResponseObjectField,
        ResponseObjectRef, ResponseObjectSet, ResponseValue, ResponseValueId,
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
    pub(super) common_update: Option<CommonUpdate>,
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
            common_update: None,
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

    pub fn insert(&mut self, id: ParentObjectId, update: ObjectUpdate) {
        self.updates[usize::from(id)] = update;
    }

    pub fn insert_subgraph_failure(&mut self, shape_id: ConcreteShapeId, error: GraphqlError) {
        let ctx = OperationPlanContext {
            schema: self.schema,
            cached: &self.operation.cached,
            plan: &self.operation.plan,
        };
        let shape = shape_id.walk(ctx);

        let mut propagate_null = false;
        let mut location_and_key = None;
        let mut fields = Vec::new();
        for field_shape in shape.fields() {
            if field_shape.key.query_position.is_none() {
                continue;
            };
            let field = field_shape
                .partition_field()
                .as_data()
                .expect("We shouldn't generate errors for lookup fields");
            location_and_key.get_or_insert_with(|| (field.location, field.response_key));
            if propagate_null | field_shape.wrapping.is_required() {
                propagate_null = true;
                continue;
            }
            fields.push(ResponseObjectField {
                key: field_shape.key,
                value: ResponseValue::Null,
            })
        }
        if let Some(first_typename_shape) = shape.typename_shapes().next() {
            if let ObjectIdentifier::Known(object_id) = shape.identifier {
                let name_id = object_id.walk(self.schema).name_id;
                fields.extend(shape.typename_shapes().map(|shape| ResponseObjectField {
                    key: shape.key,
                    value: name_id.into(),
                }))
            } else {
                propagate_null = true;
                if location_and_key.is_none() {
                    location_and_key = Some((first_typename_shape.location, first_typename_shape.key.response_key));
                }
            }
        }

        self.common_update = Some(
            if let Some(((location, key), first_parent_object)) =
                location_and_key.zip(self.parent_objects.iter().next())
            {
                self.errors.push(
                    error
                        .with_location(location)
                        .with_path((&first_parent_object.path, key)),
                );
                if propagate_null {
                    CommonUpdate::PropagateNull
                } else {
                    fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                    CommonUpdate::DefaultFields(fields.clone())
                }
            } else {
                CommonUpdate::Skip
            },
        )
    }

    pub fn insert_errors(&mut self, error: impl Into<GraphqlError>, ids: impl IntoIterator<Item = ParentObjectId>) {
        let error: GraphqlError = error.into();
        for id in ids {
            self.insert(id, ObjectUpdate::Error(error.clone()));
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

pub(crate) enum CommonUpdate {
    PropagateNull,
    DefaultFields(Vec<ResponseObjectField>),
    Skip,
}
