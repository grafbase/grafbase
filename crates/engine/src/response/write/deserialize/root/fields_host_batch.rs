use error::GraphqlError;
use runtime::extension::{Data, Response};
use serde::de::DeserializeSeed as _;

use crate::{
    prepare::{ConcreteShape, DataOrLookupFieldId, SubgraphField},
    response::{
        ParentObjectId, ParentObjectSet, ResponseField, ResponseObjectRef, ResponseValue, ResponseValueId, SeedState,
        write::deserialize::{error::DeserError, field::FieldSeed, object::ConcreteShapeFieldsContext},
    },
};

impl<'ctx, 'parent> SeedState<'ctx, 'parent> {
    pub fn ingest_fields_host_batched(
        &self,
        parent_objects: &'parent ParentObjectSet,
        fields_count: usize,
        field_results: impl IntoIterator<Item = (DataOrLookupFieldId, ParentObjectId, Response)>,
    ) {
        let object_shape = self.root_shape.concrete_shape();
        let mut batch_response_fields = vec![Vec::with_capacity(fields_count); parent_objects.len()];

        for (partition_field_id, parent_object_id, response) in field_results {
            let parent_object = &parent_objects[parent_object_id];
            let response_fields = &mut batch_response_fields[usize::from(parent_object_id)];
            self.ingest_field_into_response_fields(
                object_shape,
                partition_field_id,
                parent_object,
                response_fields,
                response,
            )
        }

        let ctx = ConcreteShapeFieldsContext::new(self, object_shape);
        for (parent_object, mut response_fields) in parent_objects.into_iter().zip(batch_response_fields) {
            ctx.finalize_deserialized_object_fields(parent_object.id, &mut response_fields);
            response_fields.sort_unstable_by_key(|field| field.key);

            let mut resp = self.response.borrow_mut();
            let fields_id = resp.data.push_owned_sorted_fields_by_key(response_fields);
            resp.insert_fields_update(parent_object, fields_id);
        }
    }

    pub fn ingest_subscription_field(
        &self,
        parent_object: &'parent ResponseObjectRef,
        partition_field_id: DataOrLookupFieldId,
        response: Response,
    ) {
        let object_shape = self.root_shape.concrete_shape();
        let mut response_fields = Vec::new();
        self.ingest_field_into_response_fields(
            object_shape,
            partition_field_id,
            parent_object,
            &mut response_fields,
            response,
        );

        let ctx = ConcreteShapeFieldsContext::new(self, object_shape);
        ctx.finalize_deserialized_object_fields(parent_object.id, &mut response_fields);
        response_fields.sort_unstable_by_key(|field| field.key);

        let mut resp = self.response.borrow_mut();
        let fields_id = resp.data.push_owned_sorted_fields_by_key(response_fields);
        resp.insert_fields_update(parent_object, fields_id);
    }

    fn ingest_field_into_response_fields(
        &self,
        object_shape: ConcreteShape<'ctx>,
        partition_field_id: DataOrLookupFieldId,
        parent_object: &'parent ResponseObjectRef,
        response_fields: &mut Vec<ResponseField>,
        response: Response,
    ) {
        let field = object_shape
            .fields()
            .find(|field_shape| field_shape.as_ref().id == partition_field_id)
            .unwrap();
        let key = field.key();

        self.reset(parent_object.path.as_slice());
        self.local_path_mut().push(ResponseValueId::field(
            parent_object.id,
            key,
            field.wrapping.is_nullable(),
        ));
        let seed = FieldSeed {
            state: self,
            field: field.as_ref(),
            wrapping: field.wrapping.to_mutable(),
        };
        match response {
            Response {
                data: Some(data),
                errors,
            } => {
                let result = match &data {
                    Data::Json(bytes) => seed
                        .deserialize(&mut sonic_rs::Deserializer::from_slice(bytes))
                        .map_err(DeserError::from),
                    Data::Cbor(bytes) => seed
                        .deserialize(&mut minicbor_serde::Deserializer::new(bytes))
                        .map_err(DeserError::from),
                };

                match result {
                    Ok(value) => {
                        response_fields.push(ResponseField { key, value });
                    }
                    Err(err) => {
                        response_fields.push(ResponseField {
                            key,
                            value: ResponseValue::Unexpected,
                        });
                        if !self.bubbling_up_deser_error.replace(true) && key.query_position.is_some() {
                            tracing::error!(
                                "Deserialization failure of for the field '{}': {err}",
                                field.partition_field().definition()
                            );
                            let mut resp = self.response.borrow_mut();
                            let path = self.path();
                            resp.propagate_null(&path);
                            resp.errors.push(
                                GraphqlError::invalid_subgraph_response()
                                    .with_path(self.path())
                                    .with_location(field.partition_field().location()),
                            );
                        }
                    }
                };

                if key.query_position.is_some() {
                    let mut resp = self.response.borrow_mut();
                    for err in errors {
                        resp.errors.push(
                            err.with_path(self.path())
                                .with_location(field.partition_field().location()),
                        );
                    }
                }
            }
            Response { data: None, errors } => {
                if field.wrapping.is_nullable() {
                    response_fields.push(ResponseField {
                        key,
                        value: ResponseValue::Null,
                    });
                    if key.query_position.is_some() {
                        let mut resp = self.response.borrow_mut();
                        for err in errors {
                            resp.errors.push(
                                err.with_path(self.path())
                                    .with_location(field.partition_field().location()),
                            );
                        }
                    }
                } else {
                    response_fields.push(ResponseField {
                        key,
                        value: ResponseValue::Unexpected,
                    });
                    if key.query_position.is_some() {
                        let mut resp = self.response.borrow_mut();
                        let path = self.path();
                        resp.propagate_null(&path);
                        for err in errors {
                            resp.errors.push(
                                err.with_path(self.path())
                                    .with_location(field.partition_field().location()),
                            );
                        }
                    }
                }
            }
        };
        self.local_path_mut().pop();
    }

    // LEGACY
    pub fn ingest_fields(
        &self,
        parent_object: &'parent ResponseObjectRef,
        field_results: impl IntoIterator<Item = (SubgraphField<'ctx>, Result<Data, GraphqlError>)>,
    ) {
        let object_shape = self.root_shape.concrete_shape();
        self.reset(parent_object.path.as_slice());

        let mut response_fields = Vec::new();
        for (partition_field, result) in field_results {
            let field = object_shape
                .fields()
                .find(|field_shape| field_shape.as_ref().id == partition_field.id)
                .unwrap();
            let seed = FieldSeed {
                state: self,
                field: field.as_ref(),
                wrapping: field.wrapping.to_mutable(),
            };
            let key = field.key();
            self.local_path_mut().push(ResponseValueId::field(
                parent_object.id,
                key,
                field.wrapping.is_nullable(),
            ));
            match result {
                Ok(data) => {
                    let result = match &data {
                        Data::Json(bytes) => seed
                            .deserialize(&mut sonic_rs::Deserializer::from_slice(bytes))
                            .map_err(DeserError::from),
                        Data::Cbor(bytes) => seed
                            .deserialize(&mut minicbor_serde::Deserializer::new(bytes))
                            .map_err(DeserError::from),
                    };

                    match result {
                        Ok(value) => {
                            response_fields.push(ResponseField { key, value });
                        }
                        Err(err) => {
                            response_fields.push(ResponseField {
                                key,
                                value: ResponseValue::Unexpected,
                            });
                            if !self.bubbling_up_deser_error.replace(true) && key.query_position.is_some() {
                                tracing::error!(
                                    "Deserialization failure of for the field '{}': {err}",
                                    field.partition_field().definition()
                                );
                                let mut resp = self.response.borrow_mut();
                                let path = self.path();
                                resp.propagate_null(&path);
                                resp.errors.push(
                                    GraphqlError::invalid_subgraph_response()
                                        .with_path(self.path())
                                        .with_location(partition_field.location()),
                                );
                            }
                        }
                    };
                }
                Err(err) => {
                    response_fields.push(ResponseField {
                        key,
                        value: ResponseValue::Unexpected,
                    });
                    if key.query_position.is_some() {
                        let mut resp = self.response.borrow_mut();
                        let path = self.path();
                        resp.propagate_null(&path);
                        resp.errors
                            .push(err.with_path(self.path()).with_location(partition_field.location()));
                    }
                }
            };
            self.local_path_mut().pop();
        }

        ConcreteShapeFieldsContext::new(self, object_shape)
            .finalize_deserialized_object_fields(parent_object.id, &mut response_fields);
        response_fields.sort_unstable_by_key(|field| field.key);

        let mut resp = self.response.borrow_mut();
        let fields_id = resp.data.push_owned_sorted_fields_by_key(response_fields);
        resp.insert_fields_update(parent_object, fields_id);
    }
}
