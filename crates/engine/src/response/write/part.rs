use id_newtypes::IdRange;
use schema::Schema;
use walker::Walk as _;

use crate::{
    prepare::{DefaultFieldShapeId, OnRootFieldsError, PreparedOperation, ResponseObjectSetId, RootFieldsShapeId},
    response::{
        DataPart, ErrorPartBuilder, GraphqlError, ResponseFieldsSortedByKey, ResponseObjectId, ResponseObjectRef,
        ResponseObjectSet, ResponsePath, ResponseValueId,
    },
};

use super::SeedState;

#[derive(id_derives::IndexedFields)]
pub(crate) struct ResponsePartBuilder<'ctx> {
    pub(super) schema: &'ctx Schema,
    pub(super) operation: &'ctx PreparedOperation,
    pub data: DataPart,
    pub errors: ErrorPartBuilder<'ctx>,
    pub(super) propagated_null_up_to_root: bool,
    pub(super) propagated_null_at: Vec<ResponseValueId>,
    pub(super) object_updates: Vec<ObjectUpdate>,
    pub(super) object_sets: Vec<(ResponseObjectSetId, ResponseObjectSet)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, id_derives::Id)]
pub struct FieldsUpdateId(u32);

impl<'ctx> ResponsePartBuilder<'ctx> {
    pub(super) fn new(schema: &'ctx Schema, operation: &'ctx PreparedOperation, data: DataPart) -> Self {
        let errors = ErrorPartBuilder::new(operation);
        Self {
            schema,
            operation,
            data,
            errors,
            object_updates: Vec::new(),
            propagated_null_up_to_root: false,
            propagated_null_at: Vec::new(),
            object_sets: Vec::new(),
        }
    }

    pub fn into_seed_state<'parent>(self, shape_id: RootFieldsShapeId) -> SeedState<'ctx, 'parent> {
        SeedState::new(self, shape_id)
    }

    pub fn propagate_null(&mut self, (parent_path, local_path): &(impl ResponsePath, impl ResponsePath)) {
        if let Some(value_id) = local_path.iter().rev().find(|value| value.is_nullable()) {
            // We can't immediately mark the value as inaccessible. Error propagation depends on
            // what the user requested directly, but we also retrieve extra fields as requirements
            // for further plans. If we were to propagate immediately while de-serializing, we
            // would skip those extra fields leading to parent field failures. Furthermore, at the
            // time we call this function, we haven't inserted the data yet into the response. It's
            // in a temporary buffer. So adding null immediately would force us to deal with
            // merges.
            self.propagated_null_at.push(*value_id)
        } else {
            self.propagate_null_parent_path(parent_path);
        }
    }

    pub fn propagate_null_parent_path(&mut self, path: &impl ResponsePath) {
        let Some(value_id) = path.iter().rev().find(|value| value.is_nullable()) else {
            self.propagated_null_up_to_root = true;
            return;
        };

        self.propagated_null_at.push(*value_id)
    }

    pub fn insert_fields_update(
        &mut self,
        parent_object: &ResponseObjectRef,
        fields: impl Into<ResponseFieldsSortedByKey>,
    ) {
        self.object_updates
            .push(ObjectUpdate::Fields(parent_object.id, fields.into()));
    }

    pub fn insert_empty_update(&mut self, parent_object: &ResponseObjectRef, shape_id: RootFieldsShapeId) {
        let shape = shape_id.walk((self.schema, self.operation));
        match shape.on_error {
            OnRootFieldsError::PropagateNull { .. } => {
                self.propagate_null_parent_path(&parent_object.path);
            }
            OnRootFieldsError::Default {
                fields_sorted_by_key, ..
            } => {
                self.object_updates
                    .push(ObjectUpdate::Default(parent_object.id, fields_sorted_by_key));
            }
            OnRootFieldsError::Skip => {}
        }
    }

    pub fn insert_empty_updates<'a>(
        &mut self,
        parent_objects: impl IntoIterator<
            IntoIter: ExactSizeIterator<Item = &'a ResponseObjectRef>,
            Item = &'a ResponseObjectRef,
        >,
        shape_id: RootFieldsShapeId,
    ) {
        let parent_objects = parent_objects.into_iter();
        let shape = shape_id.walk((self.schema, self.operation));
        match shape.on_error {
            OnRootFieldsError::PropagateNull { .. } => {
                self.propagated_null_at.reserve(parent_objects.len());
                for parent_object in parent_objects {
                    self.propagate_null_parent_path(&parent_object.path);
                }
            }
            OnRootFieldsError::Default {
                fields_sorted_by_key, ..
            } => {
                self.object_updates.reserve(parent_objects.len());
                for parent_object in parent_objects {
                    self.object_updates
                        .push(ObjectUpdate::Default(parent_object.id, fields_sorted_by_key));
                }
            }
            OnRootFieldsError::Skip => {}
        }
    }

    pub fn insert_propagated_empty_update(&mut self, parent_object: &ResponseObjectRef, shape_id: RootFieldsShapeId) {
        let shape = shape_id.walk((self.schema, self.operation));
        match shape.on_error {
            OnRootFieldsError::Default {
                fields_sorted_by_key, ..
            } => {
                self.object_updates
                    .push(ObjectUpdate::Default(parent_object.id, fields_sorted_by_key));
            }
            OnRootFieldsError::Skip | OnRootFieldsError::PropagateNull { .. } => {}
        }
    }

    pub fn insert_error_update(
        &mut self,
        parent_object: &ResponseObjectRef,
        shape_id: RootFieldsShapeId,
        errors: impl IntoIterator<Item = GraphqlError>,
    ) {
        let shape = shape_id.walk((self.schema, self.operation));
        match shape.on_error {
            OnRootFieldsError::PropagateNull {
                error_location_and_key: (location, key),
            } => {
                for err in errors {
                    self.errors
                        .push(err.with_location(location).with_path((&parent_object.path, key)));
                }
                self.propagate_null_parent_path(&parent_object.path);
            }
            OnRootFieldsError::Default {
                error_location_and_key: (location, key),
                fields_sorted_by_key,
            } => {
                for err in errors {
                    self.errors
                        .push(err.with_location(location).with_path((&parent_object.path, key)));
                }
                self.object_updates
                    .push(ObjectUpdate::Default(parent_object.id, fields_sorted_by_key));
            }
            OnRootFieldsError::Skip => {}
        }
    }

    pub fn insert_error_updates<'a>(
        &mut self,
        parent_objects: impl IntoIterator<
            IntoIter: ExactSizeIterator<Item = &'a ResponseObjectRef>,
            Item = &'a ResponseObjectRef,
        >,
        shape_id: RootFieldsShapeId,
        errors: impl IntoIterator<Item = GraphqlError>,
    ) {
        let mut parent_objects = parent_objects.into_iter();
        let shape = shape_id.walk((self.schema, self.operation));
        if let Some(first_parent_object) = parent_objects.next() {
            match shape.on_error {
                OnRootFieldsError::PropagateNull {
                    error_location_and_key: (location, key),
                } => {
                    for err in errors {
                        self.errors
                            .push(err.with_location(location).with_path((&first_parent_object.path, key)));
                    }
                    self.propagated_null_at.reserve(parent_objects.len() + 1);
                    self.propagate_null_parent_path(&first_parent_object.path);
                    for parent_object in parent_objects {
                        self.propagate_null_parent_path(&parent_object.path);
                    }
                }
                OnRootFieldsError::Default {
                    fields_sorted_by_key,
                    error_location_and_key: (location, key),
                } => {
                    for err in errors {
                        self.errors
                            .push(err.with_location(location).with_path((&first_parent_object.path, key)));
                    }
                    self.object_updates.reserve(parent_objects.len() + 1);
                    self.object_updates
                        .push(ObjectUpdate::Default(first_parent_object.id, fields_sorted_by_key));
                    for parent_object in parent_objects {
                        self.object_updates
                            .push(ObjectUpdate::Default(parent_object.id, fields_sorted_by_key));
                    }
                }
                OnRootFieldsError::Skip => {}
            }
        }
    }

    pub fn insert_errors(
        &mut self,
        parent_object: &ResponseObjectRef,
        shape_id: RootFieldsShapeId,
        errors: impl IntoIterator<Item = GraphqlError>,
    ) {
        let shape = shape_id.walk((self.schema, self.operation));
        match shape.on_error {
            OnRootFieldsError::PropagateNull {
                error_location_and_key: (location, key),
            } => {
                for err in errors {
                    self.errors
                        .push(err.with_location(location).with_path((&parent_object.path, key)));
                }
            }
            OnRootFieldsError::Default {
                error_location_and_key: (location, key),
                ..
            } => {
                for err in errors {
                    self.errors
                        .push(err.with_location(location).with_path((&parent_object.path, key)));
                }
            }
            OnRootFieldsError::Skip => {}
        }
    }

    pub fn push_object_ref(&mut self, set_id: ResponseObjectSetId, obj: ResponseObjectRef) {
        if let Some((_, set)) = self.object_sets.iter_mut().find(|(id, _)| set_id == *id) {
            set.push(obj);
        } else {
            self.object_sets.push((set_id, vec![obj]));
        }
    }
}

#[derive(Clone)]
pub(crate) enum ObjectUpdate {
    Fields(ResponseObjectId, ResponseFieldsSortedByKey),
    Default(ResponseObjectId, IdRange<DefaultFieldShapeId>),
}
