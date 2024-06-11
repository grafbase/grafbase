//! Handles merging cache responses into the OutputStore

use super::{
    engine_response::InitialOutput,
    shapes::{ConcreteShape, Field, ObjectShape, OutputShapes},
    store::{ObjectId, ValueId, ValueRecord},
};

impl<'a> InitialOutput<'a> {
    pub fn merge_cache_entry(&mut self, json: serde_json::Value, shapes: &'a OutputShapes) {
        let Some(root_object_id) = self.store.root_object() else {
            // Presumably an error bubbled up to the root, so not much we can do here.
            return;
        };

        let root_object_shape = shapes.root();
        let serde_json::Value::Object(mut object) = json else {
            todo!("something");
        };

        self.merge_cache_object(&mut object, root_object_id, root_object_shape);
    }

    fn merge_cache_object(
        &mut self,
        source_object: &mut serde_json::Map<String, serde_json::Value>,
        dest_object_id: ObjectId,
        object_shape: ConcreteShape<'a>,
    ) {
        for (name, value) in source_object {
            let Some(field_shape) = object_shape.field(name) else {
                continue;
            };
            if self.field_is_deferred(field_shape) {
                // If this field is deferred we leave it in the `serde_json::Value`
                // for later.
                return;
            }

            let field_id = self.store.field_value_id(dest_object_id, field_shape.index());

            self.merge_value(value, field_id, field_shape);
        }
    }

    fn merge_value(&mut self, value: &mut serde_json::Value, dest_id: ValueId, current_field_shape: Field<'a>) {
        let existing_value = self.store.value(dest_id);
        match (existing_value, value) {
            (ValueRecord::Unset, value) => {
                self.insert_value(value, dest_id, current_field_shape);
            }
            (ValueRecord::Null, _) => {
                // An explicit null means an error has bubbled up to this field
                // in the response, so we should ignore this part of the cached entry
            }
            (ValueRecord::List(dest_ids), serde_json::Value::Array(src_values))
                if dest_ids.len() == src_values.len() =>
            {
                for (src, dest_id) in src_values.iter_mut().zip(*dest_ids) {
                    self.merge_value(src, dest_id, current_field_shape);
                }
            }
            (ValueRecord::List(_dest_list), serde_json::Value::Array(_src_list)) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
            (ValueRecord::List(_), _) => todo!("this is a problem"),
            (ValueRecord::Object(dest_object_id), serde_json::Value::Object(source_object)) => {
                match current_field_shape.subselection_shape() {
                    Some(ObjectShape::Concrete(shape)) => {
                        self.merge_cache_object(source_object, *dest_object_id, shape)
                    }
                    Some(ObjectShape::Polymorphic(_)) => {
                        todo!("deal with polymorphic shapes");
                    }
                    None => todo!("errors innit"),
                }
            }
            (ValueRecord::Object(_), _) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
            (_, _) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
        };
    }

    /// Inserts a heirarchy of values into an empty slot in the OutputStore
    fn insert_value(&mut self, value: &mut serde_json::Value, dest_id: ValueId, field_shape: Field<'_>) {
        if self.field_is_deferred(field_shape) {
            return;
        }

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
                    self.insert_value(value, dest_id, field_shape)
                }
            }
            serde_json::Value::Object(source_object) => {
                let dest_object_shape = match field_shape.subselection_shape() {
                    Some(ObjectShape::Concrete(shape)) => shape,
                    Some(ObjectShape::Polymorphic(_)) => todo!("GB-6949"),
                    None => todo!(),
                };
                let dest_object_id = self.store.insert_object(dest_object_shape);

                self.store.write_value(dest_id, ValueRecord::Object(dest_object_id));

                for (name, value) in source_object {
                    let Some(field_shape) = dest_object_shape.field(name) else {
                        continue;
                    };

                    let field_id = self.store.field_value_id(dest_object_id, field_shape.index());

                    self.insert_value(value, field_id, field_shape);
                }
            }
            _ => todo!("this is likely an error"),
        }
    }

    fn field_is_deferred(&self, field: Field<'a>) -> bool {
        let Some(defer_label) = field.defer_label() else {
            return false;
        };

        !self.active_defers.contains(&defer_label)
    }
}

fn find_typename<'a>(
    _src_object: &'a serde_json::Map<String, serde_json::Value>,
    _current_field_shape: &Field<'_>,
) -> Option<&'a str> {
    todo!("copy the logic from engine_response")
}
