use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    fmt,
};

use schema::{DataType, ObjectId, Wrapping};
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::{
        CollectedSelectionSet, ConcreteField, ConcreteType, ExpectedSelectionSet, ExpectedType, ExtraSelectionSetId,
        PossibleField, UndeterminedSelectionSetId,
    },
    request::{BoundFieldId, FlatTypeCondition, SelectionSetType},
    response::{
        write::deserialize::{key::Key, CollectedFieldsSeed, SeedContextInner},
        ResponseEdge, ResponseKey, ResponseObject, ResponsePath,
    },
};

use super::ObjectIdentifier;

pub(crate) struct UndeterminedFieldsSeed<'ctx, 'parent> {
    pub path: &'parent ResponsePath,
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub ty: SelectionSetType,
    pub selection_set_ids: Cow<'parent, [UndeterminedSelectionSetId]>,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for UndeterminedFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for UndeterminedFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut identifier = ObjectIdentifier::new(self.ctx, self.ty);
        // Ideally we should never have an UndeterminedSelectionSet with a known object id, it
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
    expected_key: String,
    ty: ExpectedTypeCollector<UndeterminedSelectionSetId>,
    wrapping: Wrapping,
}

struct GroupForExtraField {
    edge: ResponseEdge,
    expected_key: String,
    ty: ExpectedTypeCollector<ExtraSelectionSetId>,
    wrapping: Wrapping,
}

enum ExpectedTypeCollector<Id> {
    Scalar(DataType),
    SelectionSet {
        ty: SelectionSetType,
        selection_set_ids: Vec<Id>,
    },
}

#[derive(Default)]
struct FieldsCollector {
    fields: HashMap<ResponseKey, GroupForResponseKey>,
    typename_fields: HashMap<ResponseKey, ResponseEdge>,
    // Contrary to query fields & typename, extra fields ResponseEdge is only composed of the FieldId
    // rather than position + key, so it's appropriate to aggregate on it.
    extra_fields: HashMap<ResponseEdge, GroupForExtraField>,
}

impl<'ctx, 'parent> UndeterminedFieldsSeed<'ctx, 'parent> {
    fn deserialize_concrete_object<'de, A>(self, object_id: ObjectId, map: A) -> Result<ResponseObject, A::Error>
    where
        A: MapAccess<'de>,
    {
        let exected = self.collect_fields(object_id, &self.selection_set_ids);
        CollectedFieldsSeed {
            path: self.path,
            ctx: self.ctx,
            expected: &exected,
        }
        .visit_map(map)
    }

    fn does_type_condition_apply(&self, type_condition: &Option<FlatTypeCondition>, object_id: ObjectId) -> bool {
        type_condition
            .as_ref()
            .map(|cond| cond.matches(&self.ctx.walker.schema(), object_id))
            .unwrap_or(true)
    }

    fn collect_fields(
        &self,
        object_id: ObjectId,
        selection_sets: &[UndeterminedSelectionSetId],
    ) -> CollectedSelectionSet {
        let FieldsCollector {
            fields,
            extra_fields,
            typename_fields,
        } = selection_sets
            .iter()
            .flat_map(|id| self.ctx.expectations[*id].fields.iter())
            .fold(FieldsCollector::default(), |mut acc, field| {
                match field {
                    PossibleField::TypeName { type_condition, key } => {
                        if self.does_type_condition_apply(type_condition, object_id) {
                            acc.typename_fields
                                .entry(ResponseKey::from(key))
                                .or_insert(ResponseEdge::from(*key));
                        }
                    }
                    PossibleField::Query(id) => {
                        let field = &self.ctx.expectations[*id];
                        let schema_field = self.ctx.walker.schema().walk(field.field_id);
                        if self.does_type_condition_apply(&field.type_condition, object_id) {
                            acc.fields
                                .entry(field.bound_response_key.into())
                                .and_modify(|group| {
                                    // All other cases should have been catched during validation,
                                    // inconsistent field types aren't allowed.
                                    if let ExpectedType::SelectionSet(id) = &field.ty {
                                        if let ExpectedTypeCollector::SelectionSet {
                                            ref mut selection_set_ids,
                                            ..
                                        } = group.ty
                                        {
                                            selection_set_ids.push(*id);
                                        }
                                    }
                                })
                                .or_insert_with(|| GroupForResponseKey {
                                    edge: field.bound_response_key.into(),
                                    bound_field_id: field.bound_field_id,
                                    expected_key: field.expected_key.clone(),
                                    ty: match field.ty {
                                        ExpectedType::Scalar(data_type) => ExpectedTypeCollector::Scalar(data_type),
                                        ExpectedType::SelectionSet(id) => ExpectedTypeCollector::SelectionSet {
                                            ty: SelectionSetType::maybe_from(schema_field.ty().inner().id()).unwrap(),
                                            selection_set_ids: vec![id],
                                        },
                                    },
                                    wrapping: schema_field.ty().wrapping().clone(),
                                });
                        }
                    }
                    PossibleField::Extra(id) => {
                        let field = &self.ctx.attribution[*id];
                        let schema_field = self.ctx.walker.schema().walk(field.field_id);
                        if self.does_type_condition_apply(&field.type_condition, object_id) {
                            acc.extra_fields
                                .entry(field.edge)
                                .and_modify(|group| {
                                    if let ExpectedType::SelectionSet(id) = &field.ty {
                                        if let ExpectedTypeCollector::SelectionSet {
                                            ref mut selection_set_ids,
                                            ..
                                        } = group.ty
                                        {
                                            selection_set_ids.push(*id);
                                        }
                                    }
                                })
                                .or_insert_with(|| GroupForExtraField {
                                    edge: field.edge,
                                    expected_key: field.expected_key.clone(),
                                    ty: match field.ty {
                                        ExpectedType::Scalar(data_type) => ExpectedTypeCollector::Scalar(data_type),
                                        ExpectedType::SelectionSet(id) => ExpectedTypeCollector::SelectionSet {
                                            ty: SelectionSetType::maybe_from(schema_field.ty().inner().id()).unwrap(),
                                            selection_set_ids: vec![id],
                                        },
                                    },
                                    wrapping: schema_field.ty().wrapping().clone(),
                                });
                        }
                    }
                }
                acc
            });
        let mut fields = fields
            .into_values()
            .map(|group| {
                let ty = match group.ty {
                    ExpectedTypeCollector::Scalar(data_type) => ConcreteType::Scalar(data_type),
                    ExpectedTypeCollector::SelectionSet { ty, selection_set_ids } => {
                        self.merge_selection_sets(ty, selection_set_ids)
                    }
                };
                ConcreteField {
                    edge: group.edge,
                    expected_key: group.expected_key,
                    ty,
                    bound_field_id: Some(group.bound_field_id),
                    wrapping: group.wrapping,
                }
            })
            .collect::<Vec<_>>();
        fields.extend(extra_fields.into_values().map(|group| {
            let ty = match group.ty {
                ExpectedTypeCollector::Scalar(data_type) => ConcreteType::Scalar(data_type),
                ExpectedTypeCollector::SelectionSet { ty, selection_set_ids } => {
                    self.merge_extra_selection_set(ty, selection_set_ids)
                }
            };
            ConcreteField {
                edge: group.edge,
                expected_key: group.expected_key,
                ty,
                bound_field_id: None,
                wrapping: group.wrapping,
            }
        }));
        fields.sort_unstable_by(|a, b| a.expected_key.cmp(&b.expected_key));
        CollectedSelectionSet {
            ty: SelectionSetType::Object(object_id),
            boundary_ids: selection_sets
                .iter()
                .filter_map(|id| self.ctx.expectations[*id].maybe_boundary_id)
                .collect(),
            fields,
            typename_fields: typename_fields.into_values().collect(),
        }
    }

    fn merge_selection_sets(
        &self,
        ty: SelectionSetType,
        selection_set_ids: Vec<UndeterminedSelectionSetId>,
    ) -> ConcreteType {
        if let SelectionSetType::Object(object_id) = ty {
            ConcreteType::SelectionSet(ExpectedSelectionSet::Collected(
                self.collect_fields(object_id, &selection_set_ids),
            ))
        } else {
            ConcreteType::SelectionSet(ExpectedSelectionSet::MergedUndetermined { ty, selection_set_ids })
        }
    }

    // TODO: pretty much copy pasted from collect_fields(), some generics would certainly be
    // better later
    fn merge_extra_selection_set(
        &self,
        ty: SelectionSetType,
        selection_set_ids: Vec<ExtraSelectionSetId>,
    ) -> ConcreteType {
        let extra_fields = selection_set_ids
            .into_iter()
            .flat_map(|id| &self.ctx.attribution[id].fields)
            .fold(HashMap::<ResponseEdge, GroupForExtraField>::new(), |mut acc, id| {
                let field = &self.ctx.attribution[*id];
                let schema_field = self.ctx.walker.schema().walk(field.field_id);
                // Currently, `@requires` does not support type condition, and may never will.
                assert!(field.type_condition.is_none());
                acc.entry(field.edge)
                    .and_modify(|group| {
                        if let ExpectedType::SelectionSet(id) = &field.ty {
                            if let ExpectedTypeCollector::SelectionSet {
                                ref mut selection_set_ids,
                                ..
                            } = group.ty
                            {
                                selection_set_ids.push(*id);
                            }
                        }
                    })
                    .or_insert_with(|| GroupForExtraField {
                        edge: field.edge,
                        expected_key: field.expected_key.clone(),
                        ty: match field.ty {
                            ExpectedType::Scalar(data_type) => ExpectedTypeCollector::Scalar(data_type),
                            ExpectedType::SelectionSet(id) => ExpectedTypeCollector::SelectionSet {
                                ty: SelectionSetType::maybe_from(schema_field.ty().inner().id()).unwrap(),
                                selection_set_ids: vec![id],
                            },
                        },
                        wrapping: schema_field.ty().wrapping().clone(),
                    });
                acc
            });
        let mut fields = extra_fields
            .into_values()
            .map(|group| {
                let ty = match group.ty {
                    ExpectedTypeCollector::Scalar(data_type) => ConcreteType::Scalar(data_type),
                    ExpectedTypeCollector::SelectionSet { ty, selection_set_ids } => {
                        self.merge_extra_selection_set(ty, selection_set_ids)
                    }
                };
                ConcreteField {
                    edge: group.edge,
                    expected_key: group.expected_key,
                    ty,
                    bound_field_id: None,
                    wrapping: group.wrapping,
                }
            })
            .collect::<Vec<_>>();
        fields.sort_unstable_by(|a, b| a.expected_key.cmp(&b.expected_key));
        ConcreteType::SelectionSet(ExpectedSelectionSet::Collected(CollectedSelectionSet {
            ty,
            boundary_ids: Vec::with_capacity(0),
            fields,
            typename_fields: Vec::with_capacity(0),
        }))
    }
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
