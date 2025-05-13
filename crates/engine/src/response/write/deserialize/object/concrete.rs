use id_newtypes::IdRange;
use operation::PositionedResponseKey;
use schema::ObjectDefinitionId;
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, MapAccess, Unexpected, Visitor},
};
use std::{cmp::Ordering, fmt};
use walker::Walk;

use crate::{
    prepare::{
        ConcreteShape, ConcreteShapeId, DataOrLookupFieldId, FieldShapeId, FieldShapeRecord, ObjectIdentifier,
        TypenameShapeId,
    },
    response::{
        GraphqlError, ResponseObject, ResponseObjectId, ResponseObjectRef, ResponseValue, ResponseValueId,
        value::ResponseObjectField,
        write::deserialize::{SeedState, field::FieldSeed, key::Key},
    },
};

use super::derive::DeriveContext;

pub(crate) struct ConcreteShapeSeed<'ctx, 'parent, 'state> {
    state: &'state SeedState<'ctx, 'parent>,
    parent_field: &'ctx FieldShapeRecord,
    is_required: bool,
    shape_id: ConcreteShapeId,
    known_definition_id: Option<ObjectDefinitionId>,
}

impl<'ctx, 'parent, 'state> ConcreteShapeSeed<'ctx, 'parent, 'state> {
    pub fn new(
        state: &'state SeedState<'ctx, 'parent>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: ConcreteShapeId,
    ) -> Self {
        Self {
            state,
            parent_field,
            is_required,
            shape_id,
            known_definition_id: None,
        }
    }

    pub fn new_with_known_object_definition_id(
        state: &'state SeedState<'ctx, 'parent>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: ConcreteShapeId,
        object_definition_id: ObjectDefinitionId,
    ) -> Self {
        Self {
            state,
            parent_field,
            is_required,
            shape_id,
            known_definition_id: Some(object_definition_id),
        }
    }
}

impl<'de> DeserializeSeed<'de> for ConcreteShapeSeed<'_, '_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let shape = self.shape_id.walk(self.state);
        let object_id = self.state.response.borrow_mut().data.reserve_object_id();

        Ok(self.post_process_fields_seed_result(
            shape,
            object_id,
            ConcreteShapeFieldsSeed::new(self.state, shape, object_id, self.known_definition_id)
                .deserialize(deserializer)?,
        ))
    }
}

impl<'ctx> ConcreteShapeSeed<'ctx, '_, '_> {
    // later we could also support visit_struct by using the schema as the reference structure.
    pub(super) fn visit_map<'de, A>(&self, map: A) -> Result<ResponseValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        let shape = self.shape_id.walk(self.state);
        let object_id = self.state.response.borrow_mut().data.reserve_object_id();

        Ok(self.post_process_fields_seed_result(
            shape,
            object_id,
            ConcreteShapeFieldsSeed::new(self.state, shape, object_id, self.known_definition_id).visit_map(map)?,
        ))
    }

    fn post_process_fields_seed_result(
        &self,
        shape: ConcreteShape<'ctx>,
        object_id: ResponseObjectId,
        object: ObjectValue,
    ) -> ResponseValue {
        match object {
            ObjectValue::Some { definition_id, fields } => {
                let mut resp = self.state.response.borrow_mut();
                resp.data
                    .put_object(object_id, ResponseObject::new(definition_id, fields));

                if let Some(definition_id) = definition_id {
                    let path = self.state.path();
                    // If the parent field won't be sent back to the client, there is no need to bother
                    // with inaccessible.
                    if self.state.should_report_error_for(self.parent_field)
                        && definition_id.walk(self.state.schema).is_inaccessible()
                    {
                        resp.propagate_null(&path);
                    }
                    if let Some(set_id) = shape.set_id {
                        let (parent_path, local_path) = path;
                        let mut path = Vec::with_capacity(parent_path.len() + local_path.len());
                        path.extend_from_slice(parent_path);
                        path.extend_from_slice(local_path.as_ref());
                        resp.push_object_ref(
                            set_id,
                            ResponseObjectRef {
                                id: object_id,
                                path,
                                definition_id,
                            },
                        );
                    }
                }

                object_id.into()
            }
            ObjectValue::Null => {
                if self.is_required {
                    tracing::error!(
                        "invalid type: null, expected an object at path '{}'",
                        self.state.display_path()
                    );
                    if self.state.should_report_error_for(self.parent_field) {
                        let mut resp = self.state.response.borrow_mut();
                        let path = self.state.path();
                        resp.propagate_null(&path);
                        resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(path)
                                .with_location(self.parent_field.id.walk(self.state).location()),
                        );
                    }
                    ResponseValue::Unexpected
                } else {
                    ResponseValue::Null
                }
            }
            ObjectValue::Error(error) => {
                if self.state.should_report_error_for(self.parent_field) {
                    let mut resp = self.state.response.borrow_mut();
                    let path = self.state.path();
                    // If not required, we don't need to propagate as Unexpected is equivalent to
                    // null for users.
                    if self.is_required {
                        resp.propagate_null(&path);
                    }
                    resp.errors.push(
                        error
                            .with_path(path)
                            .with_location(self.parent_field.id.walk(self.state).location()),
                    );
                }
                ResponseValue::Unexpected
            }
            ObjectValue::Unexpected => ResponseValue::Unexpected,
        }
    }
}

pub(crate) struct ConcreteShapeFieldsSeed<'ctx, 'parent, 'state> {
    state: &'state SeedState<'ctx, 'parent>,
    object_id: ResponseObjectId,
    has_error: bool,
    object_identifier: ObjectIdentifier,
    non_derived_field_shape_ids: IdRange<FieldShapeId>,
    derived_field_shape_ids: IdRange<FieldShapeId>,
    typename_shape_ids: IdRange<TypenameShapeId>,
}

impl<'ctx, 'parent, 'state> ConcreteShapeFieldsSeed<'ctx, 'parent, 'state> {
    pub fn new(
        state: &'state SeedState<'ctx, 'parent>,
        shape: ConcreteShape<'ctx>,
        object_id: ResponseObjectId,
        definition_id: Option<ObjectDefinitionId>,
    ) -> Self {
        ConcreteShapeFieldsSeed {
            state,
            object_id,
            has_error: shape.has_errors(),
            object_identifier: definition_id.map(ObjectIdentifier::Known).unwrap_or(shape.identifier),
            non_derived_field_shape_ids: IdRange {
                start: shape.field_shape_ids.start,
                end: shape.derived_field_shape_ids_start,
            },
            derived_field_shape_ids: IdRange {
                start: shape.derived_field_shape_ids_start,
                end: shape.field_shape_ids.end,
            },
            typename_shape_ids: shape.typename_shape_ids,
        }
    }
}

pub(crate) enum ObjectValue {
    Null,
    Some {
        definition_id: Option<ObjectDefinitionId>,
        fields: Vec<ResponseObjectField>,
    },
    Error(GraphqlError),
    Unexpected,
}

impl<'de> DeserializeSeed<'de> for ConcreteShapeFieldsSeed<'_, '_, '_> {
    type Value = ObjectValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl ConcreteShapeFieldsSeed<'_, '_, '_> {
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'_>>::Value {
        tracing::error!(
            "invalid type: {}, expected an object at path '{}'",
            value,
            self.state.display_path()
        );
        ObjectValue::Error(GraphqlError::invalid_subgraph_response().with_path(self.state.path()))
    }
}

impl<'de> Visitor<'de> for ConcreteShapeFieldsSeed<'_, '_, '_> {
    type Value = ObjectValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any value?")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let schema = self.state.schema;
        let mut response_fields =
            Vec::with_capacity(self.non_derived_field_shape_ids.len() + self.typename_shape_ids.len());
        let mut maybe_object_definition_id = None;

        match self.object_identifier {
            ObjectIdentifier::Known(id) => {
                maybe_object_definition_id = Some(id);
                self.visit_fields(&mut map, &mut response_fields)?;
            }
            ObjectIdentifier::Anonymous => {
                self.visit_fields(&mut map, &mut response_fields)?;
            }
            ObjectIdentifier::UnionTypename(id) => {
                if let Some(definition_id) = self.visit_fields_with_typename_detection(
                    &mut map,
                    &schema[id].possible_types_ordered_by_typename_ids,
                    &mut response_fields,
                )? {
                    maybe_object_definition_id = Some(definition_id);
                } else {
                    return Ok(ObjectValue::Error(GraphqlError::invalid_subgraph_response()));
                }
            }
            ObjectIdentifier::InterfaceTypename(id) => {
                if let Some(definition_id) = self.visit_fields_with_typename_detection(
                    &mut map,
                    &schema[id].possible_types_ordered_by_typename_ids,
                    &mut response_fields,
                )? {
                    maybe_object_definition_id = Some(definition_id);
                } else {
                    return Ok(ObjectValue::Error(GraphqlError::invalid_subgraph_response()));
                }
            }
        }

        self.post_process(&mut response_fields);

        if !self.typename_shape_ids.is_empty() {
            let Some(object_id) = maybe_object_definition_id else {
                tracing::error!(
                    "Expected to have the object definition id to generate __typename at path '{}'",
                    self.state.display_path()
                );
                return Ok(ObjectValue::Error(GraphqlError::invalid_subgraph_response()));
            };
            let name_id = schema[object_id].name_id;
            for shape in self.typename_shape_ids.walk(self.state) {
                response_fields.push(ResponseObjectField {
                    key: shape.key(),
                    value: name_id.into(),
                });
            }
        }

        Ok(ObjectValue::Some {
            definition_id: maybe_object_definition_id,
            fields: response_fields,
        })
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Signed(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Unsigned(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Float(v)))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Str(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bytes(v)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ObjectValue::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ObjectValue::Null)
    }

    fn visit_newtype_struct<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // newtype_struct are used by custom deserializers to indicate that an error happened, but
        // was already treated.
        Ok(ObjectValue::Unexpected)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // Try discarding the rest of the list, we might be able to use other parts of
        // the response.
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(self.unexpected_type(Unexpected::Seq))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data.variant::<IgnoredAny>()?;
        Ok(self.unexpected_type(Unexpected::Enum))
    }
}

impl<'ctx> ConcreteShapeFieldsSeed<'ctx, '_, '_> {
    fn post_process(&self, response_fields: &mut Vec<ResponseObjectField>) {
        let mut propagated = false;
        if self.has_error {
            let mut resp = self.state.response.borrow_mut();
            let all_fields: IdRange<FieldShapeId> =
                IdRange::from_start_and_end(self.non_derived_field_shape_ids.start, self.derived_field_shape_ids.end);
            for field_shape in all_fields.walk(self.state) {
                let mut has_errors = false;
                for error_id in field_shape.error_ids() {
                    has_errors = true;
                    let location = field_shape.as_ref().id.walk(self.state).location();
                    let path = self.state.path();
                    resp.errors
                        .push_query_error(error_id, location, (&path, field_shape.response_key));

                    if !propagated && field_shape.wrapping.is_required() {
                        propagated = true;
                        resp.propagate_null(&path);
                    }
                }
                if has_errors {
                    let key = field_shape.key();
                    if let Some(field) = response_fields.iter_mut().find(|field| field.key == key) {
                        let id = resp.data.push_inaccessible_value(std::mem::take(&mut field.value));
                        field.value = id.into();
                    } else {
                        response_fields.push(ResponseObjectField {
                            key,
                            value: ResponseValue::Null,
                        });
                    }
                }
            }
        }

        if response_fields.len() < self.non_derived_field_shape_ids.len() {
            let n = response_fields.len();
            let keys = self.state.response_keys();
            for field_shape in self.non_derived_field_shape_ids.walk(self.state) {
                if field_shape.is_absent() {
                    continue;
                }
                let key = field_shape.key();
                if !response_fields[0..n].iter().any(|field| field.key == key) {
                    if field_shape.wrapping.is_required() {
                        // If part of the query fields the user requested. We don't propagate null
                        // for extra fields.
                        if key.query_position.is_some() {
                            if key.response_key == field_shape.expected_key {
                                tracing::error!(
                                    "Error decoding response from upstream: Missing required field named '{}' at path '{}'",
                                    &keys[field_shape.expected_key],
                                    self.state.display_path()
                                )
                            } else {
                                tracing::error!(
                                    "Error decoding response from upstream: Missing required field named '{}' (expected: '{}') at path '{}'",
                                    &keys[key.response_key],
                                    &keys[field_shape.expected_key],
                                    self.state.display_path()
                                )
                            }
                            let mut resp = self.state.response.borrow_mut();
                            let path = self.state.path();
                            if !propagated {
                                propagated = true;
                                resp.propagate_null(&path);
                            }
                            resp.errors.push(
                                GraphqlError::invalid_subgraph_response()
                                    .with_path((path, key))
                                    .with_location(field_shape.as_ref().id.walk(self.state).location()),
                            );
                        }
                    } else {
                        response_fields.push(ResponseObjectField {
                            key,
                            value: ResponseValue::Null,
                        });
                    }
                }
            }
        }

        if !self.derived_field_shape_ids.is_empty() {
            let start = self.derived_field_shape_ids.start;
            let derived_field_shape_id_to_error_ids = self
                .state
                .operation
                .plan
                .query_modifications
                .field_shape_id_to_error_ids
                .as_ref();
            let mut error_ix = derived_field_shape_id_to_error_ids.partition_point(|(id, _)| *id < start);
            let mut resp = self.state.response.borrow_mut();
            let parent_path = self.state.parent_path.get();
            let mut local_path = self.state.local_path_mut();
            for field_shape in self.derived_field_shape_ids.walk(self.state) {
                // Handle any errors if there is any for this field.
                while let Some(&(id, error_id)) = derived_field_shape_id_to_error_ids.get(error_ix) {
                    match id.cmp(&field_shape.id) {
                        Ordering::Less => {
                            error_ix += 1;
                        }
                        Ordering::Equal => {
                            error_ix += 1;
                            let location = field_shape.partition_field().location();
                            let path = (parent_path, local_path.as_slice());
                            resp.errors
                                .push_query_error(error_id, location, (&path, field_shape.response_key));
                            if field_shape.wrapping.is_required() {
                                resp.propagate_null(&path);
                            }
                        }
                        Ordering::Greater => {
                            break;
                        }
                    }
                }

                if !field_shape.is_absent() {
                    let Some(shape) = field_shape.derive_entity_shape() else {
                        unreachable!("Expected to have a derive entity shape");
                    };
                    DeriveContext {
                        resp: &mut resp,
                        parent_path,
                        local_path: &mut local_path,
                        field: field_shape,
                        shape,
                    }
                    .ingest(self.object_id, response_fields);
                }
            }
        }
    }

    fn visit_fields_with_typename_detection<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        possible_types_ordered_by_typename: &[ObjectDefinitionId],
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<Option<ObjectDefinitionId>, A::Error> {
        let schema = self.state.schema;
        let keys = self.state.response_keys();
        let included_data_fields = &self
            .state
            .operation
            .plan
            .query_modifications
            .included_response_data_fields;
        let fields = &self.state.operation.cached.shapes[self.non_derived_field_shape_ids];
        let mut offset = 0;
        let mut maybe_object_definition_id: Option<ObjectDefinitionId> = None;
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            // Improves significantly (a few %) the performance to use the unchecked version.
            // SAFETY: offset is initialized 0 which always work. Later on it's only incremented by
            //         at most 1 if we find an element within [offset..]. So offset + 1 is still equal or
            //         lower than the fields length.
            if let Some(pos) = unsafe { fields.get_unchecked(offset..) }
                .iter()
                .position(|field| &keys[field.expected_key] == key)
            {
                let field = &fields[offset + pos];
                let included = match field.id {
                    DataOrLookupFieldId::Data(id) => included_data_fields[id],
                    _ => false,
                };
                self.visit_field(map, field, included, response_fields)?;
                // Each key in the JSON is unique, it's an object. So if we found it once, we won't
                // re-find it. This means that if the found field is the first one, we can increase
                // the offset to ignore for the next key.
                // Worst-case scenario if the field re-appears, we'll ignore the data.
                offset += (pos == 0) as usize;
            // This supposes that the discriminant is never part of the schema.
            } else if maybe_object_definition_id.is_none() && key == "__typename" {
                let typename = map.next_value::<&str>()?;
                maybe_object_definition_id = possible_types_ordered_by_typename
                    .binary_search_by(|probe| schema[schema[*probe].name_id].as_str().cmp(typename))
                    .map(|i| possible_types_ordered_by_typename[i])
                    .ok();
            } else {
                // Try discarding the next value, we might be able to use other parts of
                // the response.
                map.next_value::<IgnoredAny>()?;
            }
        }

        Ok(maybe_object_definition_id)
    }

    fn visit_fields<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        let keys = self.state.response_keys();
        let included_data_fields = &self
            .state
            .operation
            .plan
            .query_modifications
            .included_response_data_fields;
        let fields = &self.state.operation.cached.shapes[self.non_derived_field_shape_ids];
        let mut offset = 0;
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            // Improves significantly (a few %) the performance to use the unchecked version.
            // SAFETY: offset is initialized 0 which always work. Later on it's only incremented by
            //         at most 1 if we find an element within [offset..]. So offset + 1 is still equal or
            //         lower than the fields length.
            if let Some(pos) = unsafe { fields.get_unchecked(offset..) }
                .iter()
                .position(|field| &keys[field.expected_key] == key)
            {
                let field = &fields[offset + pos];
                let included = match field.id {
                    DataOrLookupFieldId::Data(id) => included_data_fields[id],
                    _ => false,
                };
                self.visit_field(map, field, included, response_fields)?;
                // Each key in the JSON is unique, it's an object. So if we found it once, we won't
                // re-find it. This means that if the found field is the first one, we can increase
                // the offset to ignore for the next key.
                // Worst-case scenario if the field re-appears, we'll ignore the data.
                offset += (pos == 0) as usize;
            } else {
                // Try discarding the next value, we might be able to use other parts of
                // the response.
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(())
    }

    fn visit_field<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        field: &'ctx FieldShapeRecord,
        included: bool,
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        let key = PositionedResponseKey {
            query_position: field.query_position_before_modifications,
            response_key: field.response_key,
        }
        .with_query_position_if(included);

        self.state.local_path_mut().push(ResponseValueId::Field {
            object_id: self.object_id,
            key,
            nullable: field.wrapping.is_nullable(),
        });
        let result = map.next_value_seed(FieldSeed {
            state: self.state,
            field,
            wrapping: field.wrapping.to_mutable(),
        });
        self.state.local_path_mut().pop();

        let value = result?;

        response_fields.push(ResponseObjectField { key, value });

        Ok(())
    }
}
