use std::cmp::Ordering;

use error::GraphqlError;
use walker::Walk as _;

use crate::{
    prepare::{BatchFieldShape, DerivedEntityShape, FieldShape, FieldShapeRecord},
    response::{
        ResponseListId, ResponseObject, ResponseObjectField, ResponseObjectId, ResponseObjectRef, ResponseValue,
        ResponseValueId, write::deserialize::SeedState,
    },
};

pub(super) struct DeriveContext<'ctx, 'parent, 'seed> {
    pub state: &'seed SeedState<'ctx, 'parent>,
    pub field: FieldShape<'ctx>,
    pub shape: DerivedEntityShape<'ctx>,
}

impl DeriveContext<'_, '_, '_> {
    pub fn ingest(self, parent_object_id: ResponseObjectId, response_fields: &mut Vec<ResponseObjectField>) {
        let key = self.field.key();
        self.state.local_path_mut().push(ResponseValueId::field(
            parent_object_id,
            key,
            self.field.wrapping.is_nullable(),
        ));
        let value = if let Some(batch_field) = self.shape.batch_field_shape {
            let value = response_fields
                .iter()
                .find_map(|field| {
                    if field.key.response_key == batch_field.key.response_key {
                        Some(&field.value)
                    } else {
                        None
                    }
                })
                .unwrap_or(&ResponseValue::Null);
            handle_list_derive(&self, batch_field, value)
        } else {
            let is_nullable = self.field.wrapping.is_nullable();
            handle_object_derive(&self, is_nullable, response_fields)
        };
        response_fields.push(ResponseObjectField { key, value });
        self.state.local_path_mut().pop();
    }

    pub(super) fn should_report_error_for(&self, field: &FieldShapeRecord) -> bool {
        self.state.should_report_error_for(field)
    }
}

fn handle_list_derive(
    ctx: &DeriveContext<'_, '_, '_>,
    batch_field: BatchFieldShape,
    batch_field_value: &ResponseValue,
) -> ResponseValue {
    match batch_field_value {
        ResponseValue::Null => ResponseValue::Null,
        // If a failure happened during de-serialization and we didn't report it yet
        // because it's an extra field, but this one isn't.
        ResponseValue::Unexpected => {
            if ctx.should_report_error_for(&ctx.field) {
                let mut resp = ctx.state.response.borrow_mut();
                let path = ctx.state.path();
                // If a failure happened during de-serialization and we didn't report it yet
                // because it's an extra field, but this one isn't.
                if batch_field.key.query_position.is_none() {
                    resp.errors.push(
                        GraphqlError::invalid_subgraph_response()
                            .with_path(&path)
                            .with_location(ctx.field.partition_field().location()),
                    );
                }
                if ctx.field.wrapping.is_non_null() {
                    resp.propagate_null(&path)
                }
            }
            ResponseValue::Unexpected
        }
        ResponseValue::List { id } => {
            let values =
                if let Some(scalar_field) = ctx.shape.fields().find(|field| field.shape.is_derive_from_scalar()) {
                    handle_derive_scalar_list(ctx, *id, batch_field, scalar_field)
                } else {
                    handle_derive_object_list(ctx, *id, batch_field)
                };

            ResponseValue::List {
                id: ctx.state.response.borrow_mut().data.push_list(values),
            }
        }
        _ => unreachable!(),
    }
}

fn handle_derive_scalar_list(
    ctx: &DeriveContext<'_, '_, '_>,
    id: ResponseListId,
    batch_field: BatchFieldShape,
    scalar_field: FieldShape<'_>,
) -> Vec<ResponseValue> {
    let root_definition_id = ctx.shape.object_definition_id;
    let element_is_nullable = !batch_field.wrapping.inner_is_required();

    let list = std::mem::take(&mut ctx.state.response.borrow_mut().data[id.list_id]);
    let mut derive_list = Vec::with_capacity(list.len());
    let scalar_field_key = scalar_field.key();
    if !list.is_empty() {
        ctx.state
            .local_path_mut()
            .push(ResponseValueId::index(id, 0, element_is_nullable));
        for &error_id in ctx
            .state
            .operation
            .plan
            .query_modifications
            .field_shape_id_to_error_ids
            .find_all(scalar_field.id)
        {
            let location = scalar_field.partition_field().location();
            let path = ctx.state.path();
            let mut resp = ctx.state.response.borrow_mut();
            resp.errors
                .push_query_error(error_id, location, (&path, scalar_field.response_key));
            if scalar_field.wrapping.is_non_null() {
                resp.propagate_null(&path);
            }
        }
        ctx.state.local_path_mut().pop();
    }
    for (index, value) in list.iter().enumerate() {
        ctx.state
            .local_path_mut()
            .push(ResponseValueId::index(id, index as u32, element_is_nullable));
        match value {
            ResponseValue::Null => {
                derive_list.push(ResponseValue::Null);
            }
            ResponseValue::Unexpected => {
                if ctx.should_report_error_for(&ctx.field) {
                    let path = ctx.state.path();
                    let mut resp = ctx.state.response.borrow_mut();
                    // If a failure happened during de-serialization and we didn't report it yet
                    // because it's an extra field, but this one isn't.
                    if batch_field.key.query_position.is_none() {
                        resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(&path)
                                .with_location(ctx.field.partition_field().location()),
                        );
                    }
                    if !element_is_nullable {
                        resp.propagate_null(&path)
                    }
                }
                derive_list.push(ResponseValue::Unexpected);
            }
            value => {
                let mut fields_sorted_by_key = Vec::with_capacity(ctx.shape.typename_shape_ids.len() + 1);
                fields_sorted_by_key.push(ResponseObjectField {
                    key: scalar_field_key,
                    value: value.clone(),
                });
                if fields_sorted_by_key.capacity() > 1 {
                    let name_id = ctx.shape.object_definition_id.unwrap().walk(ctx.state.schema).name_id;
                    for typename in ctx.shape.typename_shapes() {
                        fields_sorted_by_key.push(ResponseObjectField {
                            key: typename.key(),
                            value: name_id.into(),
                        });
                    }
                    fields_sorted_by_key.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                }
                let mut resp = ctx.state.response.borrow_mut();
                let id = resp.data.push_object(ResponseObject {
                    definition_id: root_definition_id,
                    fields_sorted_by_key,
                });
                if let Some(set_id) = ctx.shape.set_id {
                    let (parent_path, local_path) = ctx.state.path();
                    let mut path = Vec::with_capacity(parent_path.len() + local_path.len());
                    path.extend_from_slice(parent_path);
                    path.extend_from_slice(&local_path);
                    resp.push_object_ref(
                        set_id,
                        ResponseObjectRef {
                            id,
                            path,
                            definition_id: ctx.shape.object_definition_id.unwrap(),
                        },
                    );
                }

                derive_list.push(id.into());
            }
        }
        ctx.state.local_path_mut().pop();
    }
    ctx.state.response.borrow_mut().data[id.list_id] = list;

    derive_list
}

fn handle_derive_object_list(
    ctx: &DeriveContext<'_, '_, '_>,
    id: ResponseListId,
    batch_field: BatchFieldShape,
) -> Vec<ResponseValue> {
    let element_is_nullable = !batch_field.wrapping.inner_is_required();

    let list = std::mem::take(&mut ctx.state.response.borrow_mut().data[id.list_id]);
    let mut derive_list = Vec::with_capacity(list.len());
    for (index, value) in list.iter().enumerate() {
        ctx.state
            .local_path_mut()
            .push(ResponseValueId::index(id, index as u32, element_is_nullable));
        match value {
            ResponseValue::Null => {
                derive_list.push(ResponseValue::Null);
            }
            ResponseValue::Unexpected => {
                if ctx.should_report_error_for(&ctx.field) {
                    let path = ctx.state.path();
                    let mut resp = ctx.state.response.borrow_mut();
                    // If a failure happened during de-serialization and we didn't report it yet
                    // because it's an extra field, but this one isn't.
                    if batch_field.key.query_position.is_none() {
                        resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(&path)
                                .with_location(ctx.field.partition_field().location()),
                        );
                    }
                    if !element_is_nullable {
                        resp.propagate_null(&path)
                    }
                }
                derive_list.push(ResponseValue::Unexpected);
            }
            ResponseValue::Object { id } => {
                let mut resp = ctx.state.response.borrow_mut();
                let object = std::mem::take(&mut resp.data[id.object_id]);
                derive_list.push(handle_object_derive(
                    ctx,
                    element_is_nullable,
                    &object.fields_sorted_by_key,
                ));
                resp.data[id.object_id] = object;
            }
            _ => unreachable!(),
        }
        ctx.state.local_path_mut().pop();
    }
    ctx.state.response.borrow_mut().data[id.list_id] = list;

    derive_list
}

fn handle_object_derive(
    ctx: &DeriveContext<'_, '_, '_>,
    parent_is_nullable: bool,
    source_fields: &[ResponseObjectField],
) -> ResponseValue {
    let mut derived_response_fields = Vec::new();
    let mut is_null_entity = true;
    let first_id = ctx.shape.field_shape_ids.start;
    let derived_field_shape_id_to_error_ids = ctx
        .state
        .operation
        .plan
        .query_modifications
        .field_shape_id_to_error_ids
        .as_ref();
    let mut error_ix = derived_field_shape_id_to_error_ids.partition_point(|(id, _)| *id < first_id);
    for field in ctx.shape.fields() {
        // Handle any errors if there is any for this field.
        while let Some(&(id, error_id)) = derived_field_shape_id_to_error_ids.get(error_ix) {
            match id.cmp(&field.id) {
                Ordering::Less => {
                    error_ix += 1;
                }
                Ordering::Equal => {
                    error_ix += 1;
                    let location = field.partition_field().location();
                    let path = ctx.state.path();
                    let mut resp = ctx.state.response.borrow_mut();
                    resp.errors
                        .push_query_error(error_id, location, (&path, field.response_key));
                    if field.wrapping.is_non_null() {
                        resp.propagate_null(&path);
                    }
                }
                Ordering::Greater => {
                    break;
                }
            }
        }

        // Search for the real field.
        if let Some(ResponseObjectField { value, .. }) = source_fields
            .iter()
            .find(|source_field| source_field.key.response_key == field.expected_key)
        {
            let key = field.key();
            match value {
                ResponseValue::Null => derived_response_fields.push(ResponseObjectField {
                    key,
                    value: ResponseValue::Null,
                }),
                // If a failure happened during de-serialization and we didn't report it yet
                // because it's an extra field, but this one isn't.
                ResponseValue::Unexpected => {
                    if ctx.should_report_error_for(&field) {
                        let path = ctx.state.path();
                        let mut resp = ctx.state.response.borrow_mut();
                        if field.shape.as_derive_from_query_position().is_none() {
                            resp.errors.push(
                                GraphqlError::invalid_subgraph_response()
                                    .with_path((&path, key))
                                    .with_location(field.partition_field().location()),
                            );
                        }
                        // If not required, we don't need to propagate as Unexpected is equivalent to
                        // null for users.
                        if field.wrapping.is_non_null() {
                            resp.propagate_null(&path);
                        }
                    }

                    derived_response_fields.push(ResponseObjectField {
                        key,
                        value: ResponseValue::Unexpected,
                    })
                }
                value => {
                    is_null_entity = false;
                    derived_response_fields.push(ResponseObjectField {
                        key,
                        value: value.clone(),
                    })
                }
            };
        } else if ctx.should_report_error_for(&field) {
            // If we reached this point after handling missing values, it means the field
            // was required and an extra field. So we're not an extra field we raise an
            // error immediately. If a key field is required, the derived root field will
            // always be required.
            let path = ctx.state.path();
            let mut resp = ctx.state.response.borrow_mut();
            resp.propagate_null(&path);
            resp.errors.push(
                GraphqlError::invalid_subgraph_response()
                    .with_path((path, field.response_key))
                    .with_location(field.partition_field().location()),
            );
        }
    }

    if is_null_entity && parent_is_nullable {
        ResponseValue::Null
    } else {
        if !ctx.shape.typename_shape_ids.is_empty() {
            let name_id = ctx.shape.object_definition_id.unwrap().walk(ctx.state.schema).name_id;
            for typename in ctx.shape.typename_shapes() {
                derived_response_fields.push(ResponseObjectField {
                    key: typename.key(),
                    value: name_id.into(),
                });
            }
        }
        let mut resp = ctx.state.response.borrow_mut();
        let id = resp.data.push_object(ResponseObject::new(
            ctx.shape.object_definition_id,
            derived_response_fields,
        ));
        if let Some(set_id) = ctx.shape.set_id {
            let (parent_path, local_path) = ctx.state.path();
            let mut path = Vec::with_capacity(parent_path.len() + local_path.len());
            path.extend_from_slice(parent_path);
            path.extend_from_slice(&local_path);
            resp.push_object_ref(
                set_id,
                ResponseObjectRef {
                    id,
                    path,
                    definition_id: ctx.shape.object_definition_id.unwrap(),
                },
            );
        }
        id.into()
    }
}
