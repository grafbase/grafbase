use std::{borrow::Cow, collections::VecDeque, fmt};

use fnv::FnvHashMap;
use schema::{DataType, FieldId, ObjectId, Schema};
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::{CollectedSelectionSet, ConcreteField, ConditionalSelectionSetId, FieldType, RuntimeConcreteSelectionSet},
    request::{BoundFieldId, FlatTypeCondition, SelectionSetType},
    response::{
        write::deserialize::{key::Key, SeedContextInner},
        ResponseEdge, ResponseKey, ResponsePath, ResponseValue,
    },
};

use super::{runtime_concrete::RuntimeConcreteCollectionSetSeed, ObjectIdentifier};

pub(crate) struct ConditionalSelectionSetSeed<'ctx, 'parent> {
    pub path: &'parent ResponsePath,
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub ty: SelectionSetType,
    pub selection_set_ids: Cow<'parent, [ConditionalSelectionSetId]>,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for ConditionalSelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for ConditionalSelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut identifier = ObjectIdentifier::new(self.ctx, self.ty);
        // Ideally we should never have an ProvisionalSelectionSet with a known object id, it
        // means we could have collected fields earlier. But it can happen when a parent had
        // complex type conditions for which we couldn't collect fields.
        if let ObjectIdentifier::Known(object_id) = identifier {
            return self.deserialize_concrete_object(object_id, map);
        }
        let mut content = VecDeque::<(_, serde_value::Value)>::new();
        while let Some(key) = map.next_key::<Key<'de>>()? {
            if identifier.discriminant_key_matches(key.as_ref()) {
                identifier.determine_object_id_from_discriminant(map.next_value()?);
                return match identifier.try_into_object_id() {
                    Ok(object_id) => self.deserialize_concrete_object(
                        object_id,
                        ChainedMapAcces {
                            before: content,
                            next_value: None,
                            after: map,
                        },
                    ),
                    Err(err) => {
                        // Discarding the rest of the data.
                        while map.next_entry::<IgnoredAny, IgnoredAny>().unwrap_or_default().is_some() {}
                        Err(err)
                    }
                };
            }
            // keeping the fields until we find the actual type discriminant.
            content.push_back((key, map.next_value()?));
        }
        identifier.try_into_object_id()?;
        unreachable!(
            "if we're here it means we couldn't determine the object id, so previous statement should have returned an error."
        );
    }
}

struct GroupForResponseKey {
    edge: ResponseEdge,
    bound_field_id: BoundFieldId,
    expected_key: ResponseKey,
    schema_field_id: FieldId,
    ty: ExpectedTypeCollector<ConditionalSelectionSetId>,
}

enum ExpectedTypeCollector<Id> {
    Scalar(DataType),
    SelectionSet {
        ty: SelectionSetType,
        selection_set_ids: Vec<Id>,
    },
}

impl<'ctx, 'parent> ConditionalSelectionSetSeed<'ctx, 'parent> {
    fn deserialize_concrete_object<'de, A>(self, object_id: ObjectId, map: A) -> Result<ResponseValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        let selection_set = &self.collect_fields(object_id, &self.selection_set_ids);
        RuntimeConcreteCollectionSetSeed {
            path: self.path,
            ctx: self.ctx,
            selection_set,
        }
        .visit_map(map)
    }

    fn collect_fields(
        &self,
        object_id: ObjectId,
        selection_sets: &[ConditionalSelectionSetId],
    ) -> RuntimeConcreteSelectionSet {
        let plan = self.ctx.plan;
        let schema = plan.schema();
        let mut fields = FnvHashMap::<ResponseKey, GroupForResponseKey>::default();
        let mut typename_fields = FnvHashMap::<ResponseKey, ResponseEdge>::default();

        for selection_set_id in selection_sets {
            let selection_set = &plan[*selection_set_id];
            for (type_condition, edge) in &selection_set.typename_fields {
                if !does_type_condition_apply(&schema, type_condition, object_id) {
                    continue;
                }
                typename_fields.entry(edge.as_response_key().unwrap()).or_insert(*edge);
            }
            for field in &plan[selection_set.fields] {
                if !does_type_condition_apply(&schema, &field.type_condition, object_id) {
                    continue;
                }
                fields
                    .entry(field.edge.as_response_key().unwrap())
                    .and_modify(|group| {
                        // All other cases should have been catched during validation,
                        // inconsistent field types aren't allowed.
                        if let FieldType::SelectionSet(id) = &field.ty {
                            if let ExpectedTypeCollector::SelectionSet {
                                ref mut selection_set_ids,
                                ..
                            } = group.ty
                            {
                                selection_set_ids.push(*id);
                            }
                        }
                        if field.edge < group.edge {
                            group.edge = field.edge;
                        }
                    })
                    .or_insert_with(|| GroupForResponseKey {
                        edge: field.edge,
                        bound_field_id: field.bound_field_id,
                        expected_key: field.expected_key,
                        schema_field_id: field.schema_field_id,
                        ty: match field.ty {
                            FieldType::Scalar(data_type) => ExpectedTypeCollector::Scalar(data_type),
                            FieldType::SelectionSet(id) => ExpectedTypeCollector::SelectionSet {
                                ty: SelectionSetType::maybe_from(schema.walk(field.schema_field_id).ty().inner().id())
                                    .unwrap(),
                                selection_set_ids: vec![id],
                            },
                        },
                    });
            }
        }
        let mut fields = fields
            .into_values()
            .map(
                |GroupForResponseKey {
                     edge,
                     bound_field_id,
                     expected_key,
                     schema_field_id,
                     ty,
                 }| {
                    let ty = match ty {
                        ExpectedTypeCollector::Scalar(data_type) => FieldType::Scalar(data_type),
                        ExpectedTypeCollector::SelectionSet { ty, selection_set_ids } => {
                            self.merge_selection_sets(ty, selection_set_ids)
                        }
                    };
                    let wrapping = schema.walk(schema_field_id).ty().wrapping().clone();
                    ConcreteField {
                        edge,
                        expected_key,
                        ty,
                        bound_field_id,
                        schema_field_id,
                        wrapping,
                    }
                },
            )
            .collect::<Vec<_>>();
        let keys = plan.response_keys();
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        RuntimeConcreteSelectionSet {
            ty: SelectionSetType::Object(object_id),
            boundary_ids: selection_sets
                .iter()
                .filter_map(|id| plan[*id].maybe_boundary_id)
                .collect(),
            fields,
            typename_fields: typename_fields.into_values().collect(),
        }
    }

    fn merge_selection_sets(
        &self,
        ty: SelectionSetType,
        selection_set_ids: Vec<ConditionalSelectionSetId>,
    ) -> FieldType {
        if let SelectionSetType::Object(object_id) = ty {
            FieldType::SelectionSet(CollectedSelectionSet::RuntimeConcrete(Box::new(
                self.collect_fields(object_id, &selection_set_ids),
            )))
        } else {
            FieldType::SelectionSet(CollectedSelectionSet::MergedConditionals { ty, selection_set_ids })
        }
    }
}

fn does_type_condition_apply(schema: &Schema, type_condition: &Option<FlatTypeCondition>, object_id: ObjectId) -> bool {
    type_condition
        .as_ref()
        .map(|cond| cond.matches(schema, object_id))
        .unwrap_or(true)
}

struct ChainedMapAcces<'de, A> {
    before: VecDeque<(Key<'de>, serde_value::Value)>,
    next_value: Option<serde_value::Value>,
    after: A,
}

impl<'de, A> MapAccess<'de> for ChainedMapAcces<'de, A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.before.pop_front() {
            self.next_value = Some(value);
            seed.deserialize(serde_value::ValueDeserializer::new(serde_value::Value::String(
                key.into_string(),
            )))
            .map(Some)
        } else {
            self.after.next_key_seed(seed)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.next_value.take() {
            seed.deserialize(serde_value::ValueDeserializer::new(value))
        } else {
            self.after.next_value_seed(seed)
        }
    }
}
