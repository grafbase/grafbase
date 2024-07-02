//! Handles merging cache responses into the OutputStore

use std::collections::HashSet;

use crate::{planning::defers::DeferId, TypeRelationships};

use super::{
    shapes::{ConcreteShape, Field, ObjectShape, OutputShapes},
    store::{ObjectId, ValueId, ValueRecord},
    OutputStore,
};

impl OutputStore {
    pub fn merge_cache_entry<'a>(
        &'a mut self,
        json: &mut serde_json::Value,
        shapes: &'a OutputShapes,
        active_defers: &HashSet<DeferId>,
        type_relationships: &'a dyn TypeRelationships,
    ) {
        CacheMerge {
            store: self,
            shapes,
            type_relationships,
            mode: MergeMode::All { active_defers },
        }
        .merge_cache_entry(json);
    }

    pub fn merge_specific_defer_from_cache_entry<'a>(
        &'a mut self,
        json: &mut serde_json::Value,
        shapes: &'a OutputShapes,
        defer: DeferId,
        active_nested_defers: &HashSet<DeferId>,
        type_relationships: &'a dyn TypeRelationships,
    ) {
        CacheMerge {
            store: self,
            shapes,
            type_relationships,
            mode: MergeMode::SpecificDefer {
                defer,
                active_nested_defers,
            },
        }
        .merge_cache_entry(json);
    }
}

struct CacheMerge<'a> {
    store: &'a mut OutputStore,
    shapes: &'a OutputShapes,

    type_relationships: &'a dyn TypeRelationships,

    mode: MergeMode<'a>,
}

enum MergeMode<'a> {
    /// This mode should be used when merging into the initial
    /// response.  We take all the un-deferred fields and
    /// any deferred fields that are in active_defers
    All { active_defers: &'a HashSet<DeferId> },

    /// This mode should be used when we receive a deferred payload,
    /// passing in the name of the defer we are merging
    ///
    /// In this mode we'll only merge in fields that are part of the given defer,
    /// or are part of one of the active nested defers
    SpecificDefer {
        defer: DeferId,
        active_nested_defers: &'a HashSet<DeferId>,
    },
}

impl<'a> CacheMerge<'a> {
    fn merge_cache_entry(&mut self, json: &mut serde_json::Value) {
        let Some(root_object_id) = self.store.root_object() else {
            // Presumably an error bubbled up to the root, so not much we can do here.
            return;
        };

        let root_object_shape = self.shapes.root();
        let serde_json::Value::Object(object) = json else {
            todo!("something");
        };

        self.merge_cache_object(object, root_object_id, root_object_shape, None);
    }

    fn merge_cache_object(
        &mut self,
        source_object: &mut serde_json::Map<String, serde_json::Value>,
        dest_object_id: ObjectId,
        object_shape: ConcreteShape<'a>,
        current_defer: Option<DeferId>,
    ) {
        for (name, value) in source_object {
            let Some(field_shape) = object_shape.field(name) else {
                continue;
            };

            let new_defer = field_shape.defer_id().or(current_defer);

            if self.should_skip_field(field_shape, new_defer) {
                // If this field is deferred we leave it in the `serde_json::Value`
                // for later.
                continue;
            }

            let field_id = self.store.field_value_id(dest_object_id, field_shape.index());

            self.merge_value(value, field_id, field_shape, new_defer);
        }
    }

    fn merge_value(
        &mut self,
        value: &mut serde_json::Value,
        dest_id: ValueId,
        current_field_shape: Field<'a>,
        current_defer: Option<DeferId>,
    ) {
        let existing_value = self.store.value(dest_id);
        match (existing_value, value) {
            (ValueRecord::Unset, value) => {
                self.insert_value(value, dest_id, current_field_shape, current_defer);
            }
            (ValueRecord::Null, _) => {
                // An explicit null means an error has bubbled up to this field
                // in the response, so we should ignore this part of the cached entry
            }
            (ValueRecord::List(dest_ids), serde_json::Value::Array(src_values))
                if dest_ids.len() == src_values.len() =>
            {
                for (src, dest_id) in src_values.iter_mut().zip(*dest_ids) {
                    self.merge_value(src, dest_id, current_field_shape, current_defer);
                }
            }
            (ValueRecord::List(_dest_list), serde_json::Value::Array(_src_list)) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
            (ValueRecord::List(_), _) => todo!("this is a problem"),
            (ValueRecord::Object(dest_object_id), serde_json::Value::Object(source_object)) => {
                let shape_id = self.store.read_object(self.shapes, *dest_object_id).shape_id();
                let dest_object_shape = self.shapes.concrete_object(shape_id);

                self.merge_cache_object(source_object, *dest_object_id, dest_object_shape, current_defer)
            }
            (ValueRecord::Object(_), _) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
            (x, y) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens: {x:?}, {y:?}");
            }
        };
    }

    /// Inserts a heirarchy of values into an empty slot in the OutputStore
    fn insert_value(
        &mut self,
        value: &mut serde_json::Value,
        dest_id: ValueId,
        field_shape: Field<'_>,
        current_defer: Option<DeferId>,
    ) {
        if field_shape.is_leaf() {
            match std::mem::take(value) {
                serde_json::Value::Null => self.store.write_value(dest_id, ValueRecord::Null),
                serde_json::Value::Bool(inner) => self.store.write_value(dest_id, ValueRecord::Boolean(inner)),
                serde_json::Value::Number(inner) => self.store.write_value(dest_id, ValueRecord::Number(inner)),
                serde_json::Value::String(inner) => self.store.write_value(dest_id, ValueRecord::String(inner.into())),
                value @ (serde_json::Value::Array(_) | serde_json::Value::Object(_)) => self
                    .store
                    .write_value(dest_id, ValueRecord::InlineValue(Box::new(value.into()))),
            }
            return;
        }

        match value {
            serde_json::Value::Null => self.store.write_value(dest_id, ValueRecord::Null),
            serde_json::Value::Array(list) => {
                let dest_ids = self.store.new_list(list.len());
                self.store.write_value(dest_id, ValueRecord::List(dest_ids));

                for (value, dest_id) in list.iter_mut().zip(dest_ids) {
                    self.insert_value(value, dest_id, field_shape, current_defer)
                }
            }
            serde_json::Value::Object(source_object) => {
                let dest_object_shape = match field_shape.subselection_shape() {
                    Some(ObjectShape::Concrete(shape)) => shape,
                    Some(ObjectShape::Polymorphic(shape)) => {
                        let Some(typename) = source_object.get("__typename").and_then(|value| value.as_str()) else {
                            todo!("GB-6966")
                        };
                        shape.concrete_shape_for_typename(typename, self.type_relationships)
                    }
                    None => todo!("GB-6966"),
                };
                let dest_object_id = self.store.insert_object(dest_object_shape);

                self.store.write_value(dest_id, ValueRecord::Object(dest_object_id));

                for (name, value) in source_object {
                    let Some(field_shape) = dest_object_shape.field(name) else {
                        continue;
                    };
                    let new_defer_label = field_shape.defer_id().or(current_defer);

                    if self.should_skip_field(field_shape, new_defer_label) {
                        continue;
                    }

                    let field_id = self.store.field_value_id(dest_object_id, field_shape.index());

                    self.insert_value(value, field_id, field_shape, new_defer_label);
                }
            }
            _ => todo!("this is likely an error"),
        }
    }

    fn should_skip_field(&self, field: Field<'a>, current_defer: Option<DeferId>) -> bool {
        match self.mode {
            MergeMode::All { active_defers } => {
                let Some(defer) = field.defer_id() else {
                    return false;
                };
                !active_defers.contains(&defer)
            }
            MergeMode::SpecificDefer {
                defer,
                active_nested_defers,
            } => {
                if let Some(field_defer_id) = field.defer_id() {
                    // If this field is unique to a defer, it needs to be one of the defers
                    // we care about
                    return field_defer_id != defer && !active_nested_defers.contains(&field_defer_id);
                }
                if field.is_leaf() {
                    let Some(current_defer) = current_defer else {
                        return true;
                    };
                    current_defer != defer && !active_nested_defers.contains(&current_defer)
                } else {
                    false
                }
            }
        }
    }
}
