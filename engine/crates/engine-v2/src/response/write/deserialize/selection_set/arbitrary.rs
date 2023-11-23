use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

use schema::{DataType, ObjectId};
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::{
        ExpectedArbitraryFields, ExpectedGoupedField, ExpectedGroupedFields, ExpectedSelectionSet, ExpectedType,
        FieldOrTypeName,
    },
    request::{BoundFieldId, SelectionSetRoot},
    response::{
        write::deserialize::{ObjectFieldsSeed, SeedContext},
        BoundResponseKey, ResponseKey, ResponseObject, ResponsePath,
    },
};

pub struct ArbitraryFieldsSeed<'ctx, 'parent> {
    pub path: &'parent ResponsePath,
    pub ctx: &'parent SeedContext<'ctx>,
    pub expected: &'parent ExpectedArbitraryFields,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for ArbitraryFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for ArbitraryFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut identifier = super::ObjectIdentifier::new(self.ctx, self.expected.root);
        let mut content = VecDeque::<(&str, serde_value::Value)>::new();
        while let Some(key) = map.next_key::<&str>()? {
            if identifier.discriminant_key_matches(key) {
                identifier.determine_object_id_from_discriminant(map.next_value()?);
                return match identifier.try_into_object_id() {
                    Ok(object_id) => self.deserialize_concrete_object(object_id, content, map),
                    Err(err) => {
                        // Discarding the rest of the data.
                        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                        Err(err)
                    }
                };
            }
            // keeping the fields until we find the actual type discriminant.
            content.push_back((key, map.next_value()?));
        }
        identifier.try_into_object_id()?;
        unreachable!(
            "if we're here it means we couldn't determine the object id, so previous statement return an error."
        );
    }
}

struct GroupForResponseKey<'a> {
    key: BoundResponseKey,
    bound_field_id: BoundFieldId,
    expected_name: &'a str,
    expected: ExpectedTypeCollector<'a>,
}

enum ExpectedTypeCollector<'a> {
    Scalar(DataType),
    Object(Vec<&'a ExpectedArbitraryFields>),
}

#[derive(Default)]
struct FieldsCollector<'a> {
    fields: HashMap<ResponseKey, GroupForResponseKey<'a>>,
    typename_fields: Vec<BoundResponseKey>,
}

impl<'ctx, 'parent> ArbitraryFieldsSeed<'ctx, 'parent> {
    fn deserialize_concrete_object<'de, A>(
        self,
        object_id: ObjectId,
        content: VecDeque<(&'de str, serde_value::Value)>,
        map: A,
    ) -> Result<ResponseObject, A::Error>
    where
        A: MapAccess<'de>,
    {
        let exected = self.collect_fields(object_id, vec![&self.expected]);
        ObjectFieldsSeed {
            path: self.path,
            ctx: self.ctx,
            expected: &exected,
        }
        .visit_map(ChainedMapAcces {
            before: content,
            after: map,
        })
    }

    fn collect_fields(
        &self,
        object_id: ObjectId,
        selection_sets: Vec<&'parent ExpectedArbitraryFields>,
    ) -> ExpectedGroupedFields {
        let FieldsCollector {
            fields,
            typename_fields,
        } = selection_sets
            .into_iter()
            .flat_map(|selection_set| selection_set.fields.iter())
            .fold(FieldsCollector::default(), |mut acc, field| {
                if field
                    .type_condition
                    .as_ref()
                    .map(|cond| cond.matches(&self.ctx.schema_walker, object_id))
                    .unwrap_or(true)
                {
                    let key = self.ctx.operation[field.bound_field_id].bound_response_key;
                    if let Some(ref expected_name) = field.expected_name {
                        acc.fields
                            .entry(key.into())
                            .and_modify(|group| {
                                // All other cases should have been catched during validation,
                                // inconsistent field types aren't allowed.
                                if let ExpectedType::Object(selection_set) = &field.ty {
                                    if let ExpectedTypeCollector::Object(ref mut selection_sets) = group.expected {
                                        selection_sets.push(selection_set);
                                    }
                                }
                            })
                            .or_insert_with(|| GroupForResponseKey {
                                key,
                                bound_field_id: field.bound_field_id,
                                expected_name: expected_name.as_str(),
                                expected: match &field.ty {
                                    ExpectedType::TypeName => unreachable!(
                                        "meta fields have no expected name since they can't be provided by upstream."
                                    ),
                                    ExpectedType::Scalar(data_type) => ExpectedTypeCollector::Scalar(*data_type),
                                    ExpectedType::Object(selection_set) => {
                                        ExpectedTypeCollector::Object(vec![&selection_set])
                                    }
                                },
                            });
                    } else {
                        acc.typename_fields.push(key);
                    }
                }
                acc
            });
        let fields = fields.into_values().map(|field| {
            let bound_field = &self
                .ctx
                .operation
                .walk_field(self.ctx.schema_walker, field.bound_field_id);
            let ty = match field.expected {
                ExpectedTypeCollector::Scalar(data_type) => ExpectedType::Scalar(data_type),
                ExpectedTypeCollector::Object(selection_sets) => self.merge_selection_sets(selection_sets),
            };
            FieldOrTypeName::Field(ExpectedGoupedField {
                bound_response_key: field.key,
                expected_name: field.expected_name.to_string(),
                ty,
                definition_id: bound_field.bound_definition_id(),
                wrapping: bound_field
                    .definition()
                    .as_field()
                    .expect("not a meta field")
                    .ty()
                    .wrapping
                    .clone(),
            })
        });
        ExpectedGroupedFields::new(
            SelectionSetRoot::Object(object_id),
            fields.chain(typename_fields.into_iter().map(FieldOrTypeName::TypeName)),
        )
    }

    fn merge_selection_sets(&self, selection_sets: Vec<&'parent ExpectedArbitraryFields>) -> ExpectedType {
        // not entirely about this root.
        let root = selection_sets[0].root;
        if let SelectionSetRoot::Object(object_id) = root {
            ExpectedType::Object(Box::new(ExpectedSelectionSet::Grouped(
                self.collect_fields(object_id, selection_sets),
            )))
        } else {
            ExpectedType::Object(Box::new(ExpectedSelectionSet::Arbitrary(ExpectedArbitraryFields {
                root,
                // Should be reworked later to use references instead of cloning everything.
                fields: selection_sets
                    .into_iter()
                    .flat_map(|selection_set| selection_set.fields.iter().cloned())
                    .collect(),
            })))
        }
    }
}

struct ChainedMapAcces<'de, A> {
    before: VecDeque<(&'de str, serde_value::Value)>,
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
        if let Some(&(key, _)) = self.before.front() {
            seed.deserialize(serde_value::ValueDeserializer::new(serde_value::Value::String(
                key.to_string(),
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
        if let Some((_, value)) = self.before.pop_front() {
            seed.deserialize(serde_value::ValueDeserializer::new(value))
        } else {
            self.after.next_value_seed(seed)
        }
    }
}
