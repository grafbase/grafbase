use std::cmp::Ordering;

use error::GraphqlError;
use walker::Walk as _;

use crate::{
    prepare::{BatchFieldShape, DataOrLookupFieldId, DerivedEntityShape, FieldShape, FieldShapeRecord},
    response::{
        ResponseField, ResponseFieldsSortedByKey, ResponseListId, ResponseObject, ResponseObjectId, ResponseObjectRef,
        ResponsePartBuilder, ResponseValue, ResponseValueId,
    },
};

pub(super) struct DeriveContext<'ctx, 'parent, 'seed> {
    pub resp: &'seed mut ResponsePartBuilder<'ctx>,
    pub parent_path: &'parent [ResponseValueId],
    pub local_path: &'seed mut Vec<ResponseValueId>,
    pub field: FieldShape<'ctx>,
    pub shape: DerivedEntityShape<'ctx>,
}

impl DeriveContext<'_, '_, '_> {
    pub fn ingest(mut self, parent_object_id: ResponseObjectId, response_fields: &mut Vec<ResponseField>) {
        let key = self.field.key();
        self.local_path.push(ResponseValueId::field(
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
            handle_list_derive(&mut self, batch_field, value)
        } else {
            let is_nullable = self.field.wrapping.is_nullable();
            handle_object_derive(&mut self, is_nullable, response_fields)
        };
        response_fields.push(ResponseField { key, value });
        self.local_path.pop();
    }

    pub(super) fn should_report_error_for(&self, field: &FieldShapeRecord) -> bool {
        field.query_position_before_modifications.is_some()
            && match field.id {
                DataOrLookupFieldId::Data(id) => {
                    self.resp
                        .operation
                        .plan
                        .query_modifications
                        .included_response_data_fields[id]
                }
                DataOrLookupFieldId::Lookup(_) => false,
            }
    }
}

fn handle_list_derive(
    ctx: &mut DeriveContext<'_, '_, '_>,
    batch_field: BatchFieldShape,
    batch_field_value: &ResponseValue,
) -> ResponseValue {
    match batch_field_value {
        ResponseValue::Null => ResponseValue::Null,
        // If a failure happened during de-serialization and we didn't report it yet
        // because it's an extra field, but this one isn't.
        ResponseValue::Unexpected => {
            if ctx.should_report_error_for(&ctx.field) {
                let path = (ctx.parent_path, ctx.local_path.as_slice());
                // If a failure happened during de-serialization and we didn't report it yet
                // because it's an extra field, but this one isn't.
                if batch_field.key.query_position.is_none() {
                    ctx.resp.errors.push(
                        GraphqlError::invalid_subgraph_response()
                            .with_path(path)
                            .with_location(ctx.field.partition_field().location()),
                    );
                }
                if ctx.field.wrapping.is_non_null() {
                    ctx.resp.propagate_null(&path)
                }
            }
            ResponseValue::Unexpected
        }
        ResponseValue::List { id, offset, limit } => {
            let values =
                if let Some(scalar_field) = ctx.shape.fields().find(|field| field.shape.is_derive_from_scalar()) {
                    handle_derive_scalar_list(ctx, *id, *offset, *limit, batch_field, scalar_field)
                } else {
                    handle_derive_object_list(ctx, *id, *offset, *limit, batch_field)
                };

            let limit = values.len() as u32;
            ResponseValue::List {
                // One of the rare case where we need to be careful with shared list as we're
                // reading from one. We would need to ensure we don't retrieve the same one.
                id: ctx.resp.data.push_list(values),
                offset: 0,
                limit,
            }
        }
        _ => unreachable!(),
    }
}

fn handle_derive_scalar_list(
    ctx: &mut DeriveContext<'_, '_, '_>,
    id: ResponseListId,
    offset: u32,
    limit: u32,
    batch_field: BatchFieldShape,
    scalar_field: FieldShape<'_>,
) -> Vec<ResponseValue> {
    let root_definition_id = ctx.shape.object_definition_id;
    let element_is_nullable = !batch_field.wrapping.inner_is_required();

    let shared_list = std::mem::take(&mut ctx.resp.data[id.list_id]);
    let slice = &shared_list[offset as usize..(offset + limit) as usize];

    let mut derive_list = Vec::with_capacity(slice.len());
    let scalar_field_key = scalar_field.key();
    if !slice.is_empty() {
        ctx.local_path.push(ResponseValueId::index(id, 0, element_is_nullable));
        for &error_id in ctx
            .resp
            .operation
            .plan
            .query_modifications
            .field_shape_id_to_error_ids
            .find_all(scalar_field.id)
        {
            let location = scalar_field.partition_field().location();
            let path = (ctx.parent_path, ctx.local_path.as_slice());
            ctx.resp
                .errors
                .push_query_error(error_id, location, (&path, scalar_field.response_key));
            if scalar_field.wrapping.is_non_null() {
                ctx.resp.propagate_null(&path);
            }
        }
        ctx.local_path.pop();
    }
    for (index, value) in slice.iter().enumerate() {
        ctx.local_path
            .push(ResponseValueId::index(id, index as u32, element_is_nullable));
        match value {
            ResponseValue::Null => {
                derive_list.push(ResponseValue::Null);
            }
            ResponseValue::Unexpected => {
                if ctx.should_report_error_for(&ctx.field) {
                    let path = (ctx.parent_path, ctx.local_path.as_slice());
                    // If a failure happened during de-serialization and we didn't report it yet
                    // because it's an extra field, but this one isn't.
                    if batch_field.key.query_position.is_none() {
                        ctx.resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(path)
                                .with_location(ctx.field.partition_field().location()),
                        );
                    }
                    if !element_is_nullable {
                        ctx.resp.propagate_null(&path)
                    }
                }
                derive_list.push(ResponseValue::Unexpected);
            }
            value => {
                let mut fields_sorted_by_key = Vec::with_capacity(ctx.shape.typename_shape_ids.len() + 1);
                fields_sorted_by_key.push(ResponseField {
                    key: scalar_field_key,
                    value: value.clone(),
                });
                if fields_sorted_by_key.capacity() > 1 {
                    let name_id = ctx.shape.object_definition_id.unwrap().walk(ctx.resp.schema).name_id;
                    for typename in ctx.shape.typename_shapes() {
                        fields_sorted_by_key.push(ResponseField {
                            key: typename.key(),
                            value: name_id.into(),
                        });
                    }
                    fields_sorted_by_key.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                }
                let fields_id = ctx.resp.data.push_owned_sorted_fields_by_key(fields_sorted_by_key);
                let id = ctx
                    .resp
                    .data
                    .push_object(ResponseObject::new(root_definition_id, fields_id));
                if let Some(set_id) = ctx.shape.set_id {
                    let mut path = Vec::with_capacity(ctx.parent_path.len() + ctx.local_path.len());
                    path.extend_from_slice(ctx.parent_path);
                    path.extend_from_slice(ctx.local_path.as_ref());
                    ctx.resp.push_object_ref(
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
        ctx.local_path.pop();
    }
    ctx.resp.data[id.list_id] = shared_list;

    derive_list
}

fn handle_derive_object_list(
    ctx: &mut DeriveContext<'_, '_, '_>,
    id: ResponseListId,
    offset: u32,
    limit: u32,
    batch_field: BatchFieldShape,
) -> Vec<ResponseValue> {
    let element_is_nullable = !batch_field.wrapping.inner_is_required();

    let shared_list = std::mem::take(&mut ctx.resp.data[id.list_id]);
    let slice = &shared_list[offset as usize..(offset + limit) as usize];

    let mut derive_list = Vec::with_capacity(slice.len());
    for (index, value) in slice.iter().enumerate() {
        ctx.local_path
            .push(ResponseValueId::index(id, index as u32, element_is_nullable));
        match value {
            ResponseValue::Null => {
                derive_list.push(ResponseValue::Null);
            }
            ResponseValue::Unexpected => {
                if ctx.should_report_error_for(&ctx.field) {
                    let path = (ctx.parent_path, ctx.local_path.as_slice());
                    // If a failure happened during de-serialization and we didn't report it yet
                    // because it's an extra field, but this one isn't.
                    if batch_field.key.query_position.is_none() {
                        ctx.resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(path)
                                .with_location(ctx.field.partition_field().location()),
                        );
                    }
                    if !element_is_nullable {
                        ctx.resp.propagate_null(&path)
                    }
                }
                derive_list.push(ResponseValue::Unexpected);
            }
            ResponseValue::Object { id } => match ctx.resp.data[id.object_id].fields_sorted_by_key {
                ResponseFieldsSortedByKey::Slice {
                    fields_id,
                    offset,
                    limit,
                } => {
                    let fields = std::mem::take(&mut ctx.resp.data[fields_id]);
                    let start = offset as usize;
                    let end = start + limit as usize;
                    derive_list.push(handle_object_derive(ctx, element_is_nullable, &fields[start..end]));
                    ctx.resp.data[fields_id] = fields;
                }
                ResponseFieldsSortedByKey::Owned { fields_id } => {
                    let fields = std::mem::take(&mut ctx.resp.data[fields_id]);
                    derive_list.push(handle_object_derive(ctx, element_is_nullable, &fields));
                    ctx.resp.data[fields_id] = fields;
                }
            },
            _ => unreachable!(),
        }
        ctx.local_path.pop();
    }
    ctx.resp.data[id.list_id] = shared_list;

    derive_list
}

fn handle_object_derive(
    ctx: &mut DeriveContext<'_, '_, '_>,
    parent_is_nullable: bool,
    source_fields: &[ResponseField],
) -> ResponseValue {
    let mut derived_response_fields = Vec::new();
    let mut is_null_entity = true;
    let first_id = ctx.shape.field_shape_ids.start;
    let derived_field_shape_id_to_error_ids = ctx
        .resp
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
                    let path = (ctx.parent_path, ctx.local_path.as_slice());
                    ctx.resp
                        .errors
                        .push_query_error(error_id, location, (&path, field.response_key));
                    if field.wrapping.is_non_null() {
                        ctx.resp.propagate_null(&path);
                    }
                }
                Ordering::Greater => {
                    break;
                }
            }
        }

        // Search for the real field.
        if let Some(ResponseField { value, .. }) = source_fields
            .iter()
            .find(|source_field| source_field.key.response_key == field.expected_key)
        {
            let key = field.key();
            match value {
                ResponseValue::Null => derived_response_fields.push(ResponseField {
                    key,
                    value: ResponseValue::Null,
                }),
                // If a failure happened during de-serialization and we didn't report it yet
                // because it's an extra field, but this one isn't.
                ResponseValue::Unexpected => {
                    if ctx.should_report_error_for(&field) {
                        let path = (ctx.parent_path, ctx.local_path.as_slice());
                        if field.shape.as_derive_from_query_position().is_none() {
                            ctx.resp.errors.push(
                                GraphqlError::invalid_subgraph_response()
                                    .with_path((path, key))
                                    .with_location(field.partition_field().location()),
                            );
                        }
                        // If not required, we don't need to propagate as Unexpected is equivalent to
                        // null for users.
                        if field.wrapping.is_non_null() {
                            ctx.resp.propagate_null(&path);
                        }
                    }

                    derived_response_fields.push(ResponseField {
                        key,
                        value: ResponseValue::Unexpected,
                    })
                }
                value => {
                    is_null_entity = false;
                    derived_response_fields.push(ResponseField {
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
            let path = (ctx.parent_path, ctx.local_path.as_slice());
            ctx.resp.propagate_null(&path);
            ctx.resp.errors.push(
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
            let name_id = ctx.shape.object_definition_id.unwrap().walk(ctx.resp.schema).name_id;
            for typename in ctx.shape.typename_shapes() {
                derived_response_fields.push(ResponseField {
                    key: typename.key(),
                    value: name_id.into(),
                });
            }
        }
        derived_response_fields.sort_unstable_by_key(|field| field.key);
        let fields_id = ctx.resp.data.push_owned_sorted_fields_by_key(derived_response_fields);
        let id = ctx
            .resp
            .data
            .push_object(ResponseObject::new(ctx.shape.object_definition_id, fields_id));
        if let Some(set_id) = ctx.shape.set_id {
            let mut path = Vec::with_capacity(ctx.parent_path.len() + ctx.local_path.len());
            path.extend_from_slice(ctx.parent_path);
            path.extend_from_slice(ctx.local_path.as_ref());
            ctx.resp.push_object_ref(
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
