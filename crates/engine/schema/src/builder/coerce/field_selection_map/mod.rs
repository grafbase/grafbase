mod bind;
mod target;

pub(crate) use bind::*;
use std::collections::VecDeque;
use target::*;

use ::field_selection_map::*;
use wrapping::Wrapping;

use crate::{
    CompositeTypeId, EntityDefinitionId, FieldDefinitionId, InputValueDefinitionId, TypeDefinitionId, TypeRecord,
    builder::GraphBuilder,
};

use super::field_set::is_disjoint;

impl GraphBuilder<'_> {
    pub(crate) fn parse_field_selection_map_for_argument(
        &mut self,
        output: EntityDefinitionId,
        field_id: FieldDefinitionId,
        argument_id: InputValueDefinitionId,
        field_selection_map: &str,
    ) -> Result<BoundSelectedValue<InputValueDefinitionId>, String> {
        let selected_value = SelectedValue::try_from(field_selection_map)?;
        let wrapping = self.graph[argument_id].ty_record.wrapping;
        bind_selected_value(
            self,
            (output.into(), Wrapping::required()),
            (Input::Argument { field_id, argument_id }, wrapping),
            selected_value,
        )
    }

    pub(crate) fn parse_field_selection_map_for_field(
        &mut self,
        output: EntityDefinitionId,
        target: FieldDefinitionId,
        field_selection_map: &str,
    ) -> Result<BoundSelectedValue<FieldDefinitionId>, String> {
        let selected_value = SelectedValue::try_from(field_selection_map)?;
        let wrapping = self.graph[target].ty_record.wrapping;
        bind_selected_value(
            self,
            (output.into(), Wrapping::required()),
            (target, wrapping),
            selected_value,
        )
    }
}

fn bind_selected_value<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    output: (CompositeTypeId, Wrapping),
    target: (T, Wrapping),
    selected_value: SelectedValue<'_>,
) -> Result<BoundSelectedValue<T::Id>, String> {
    let alternatives = selected_value
        .alternatives
        .into_iter()
        .map(|entry| bind_selected_value_entry(ctx, output, target, entry))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BoundSelectedValue { alternatives })
}

fn bind_selected_value_entry<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    output: (CompositeTypeId, Wrapping),
    target: (T, Wrapping),
    selected_value: SelectedValueEntry<'_>,
) -> Result<BoundSelectedValueEntry<T::Id>, String> {
    match selected_value {
        SelectedValueEntry::Path(path) => {
            let (path, _) = bind_path(ctx, output, path)?;
            let last = *path.0.last().unwrap();
            ensure_type_compatibility(ctx, target, last)?;
            Ok(BoundSelectedValueEntry::Path(path))
        }
        SelectedValueEntry::ObjectWithPath { path, object } => {
            let (path, (output, output_wrapping)) = bind_path(ctx, output, path)?;
            let output = output
                .as_composite_type()
                .ok_or_else(|| format!("Type {} does not have any fields", ctx[ctx.definition_name_id(output)]))?;
            let object = bind_selected_object_value(ctx, (output, output_wrapping), target, object)?;
            Ok(BoundSelectedValueEntry::ObjectWithPath { path, object })
        }
        SelectedValueEntry::ListWithPath { path, list } => {
            let (path, (output, output_wrapping)) = bind_path(ctx, output, path)?;
            let output = output
                .as_composite_type()
                .ok_or_else(|| format!("Type {} does not have any fields", ctx[ctx.definition_name_id(output)]))?;
            let list = bind_selected_list_value(ctx, (output, output_wrapping), target, list)?;
            Ok(BoundSelectedValueEntry::ListWithPath { path, list })
        }
        SelectedValueEntry::Object(object) => {
            let object = bind_selected_object_value(ctx, output, target, object)?;
            Ok(BoundSelectedValueEntry::Object(object))
        }
    }
}

fn bind_path(
    ctx: &mut GraphBuilder<'_>,
    (output, mut output_wrapping): (CompositeTypeId, Wrapping),
    path: Path<'_>,
) -> Result<(BoundPath, (TypeDefinitionId, Wrapping)), String> {
    if output_wrapping.is_list() {
        return Err(format!(
            "Cannot select a field from {}, it's a list",
            ctx.type_name(TypeRecord {
                definition_id: output.into(),
                wrapping: output_wrapping
            })
        ));
    }
    let mut output: TypeDefinitionId = if let Some(ty) = path.ty {
        bind_type_condition(ctx, output, ty)?.into()
    } else {
        output.into()
    };

    let mut out = Vec::new();
    let mut segments = VecDeque::from(path.segments);
    while let Some(segment) = segments.pop_front() {
        let field_ids = match output {
            TypeDefinitionId::Interface(id) => ctx.graph[id].field_ids,
            TypeDefinitionId::Object(id) => ctx.graph[id].field_ids,
            _ => {
                return Err(format!(
                    "Type {} does not have any fields",
                    ctx[ctx.definition_name_id(output)]
                ));
            }
        };
        let field_id = field_ids
            .into_iter()
            .find(|id| ctx[ctx.graph[*id].name_id] == segment.field)
            .ok_or_else(|| {
                format!(
                    "Type {} does not have a field named '{}'",
                    ctx[ctx.definition_name_id(output)],
                    segment.field
                )
            })?;
        out.push(field_id);
        output = ctx.graph[field_id].ty_record.definition_id;
        let parent_output_is_nullable = output_wrapping.is_nullable();
        output_wrapping = ctx.graph[field_id].ty_record.wrapping;
        if parent_output_is_nullable {
            output_wrapping = output_wrapping.without_non_null();
        }

        if let Some(ty) = segment.ty {
            if let Some(id) = output.as_composite_type() {
                output = bind_type_condition(ctx, id, ty)?.into();
            } else {
                return Err(format!(
                    "Field '{}' doesn't return an object, interface or union and thus cannot have type condition '{}'",
                    ctx[ctx.graph[field_id].name_id], ty
                ));
            }
        }
    }

    Ok((BoundPath(out), (output, output_wrapping)))
}

fn bind_type_condition(
    ctx: &mut GraphBuilder<'_>,
    parent: CompositeTypeId,
    ty: &str,
) -> Result<EntityDefinitionId, String> {
    let Ok(ty_id) = ctx
        .graph
        .type_definitions_ordered_by_name
        .binary_search_by(|probe| ctx[ctx.definition_name_id(*probe)].as_str().cmp(ty))
        .map(|ix| ctx.graph.type_definitions_ordered_by_name[ix])
    else {
        return Err(format!("Type {} does not exist", ty));
    };
    let Some(ty_id) = ty_id.as_entity() else {
        return Err(format!("Type {} is not an object or an interface", ty));
    };
    if is_disjoint(ctx, parent, ty_id.into()) {
        return Err(format!(
            "Type mismatch: Cannot use type condition {} on {}",
            ty,
            ctx[ctx.definition_name_id(parent.into())]
        ));
    }
    Ok(ty_id)
}

fn bind_selected_object_value<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    (output, output_wrapping): (CompositeTypeId, Wrapping),
    (target, target_wrapping): (T, Wrapping),
    object: SelectedObjectValue<'_>,
) -> Result<BoundSelectedObjectValue<T::Id>, String> {
    if output_wrapping.is_list() {
        return Err(format!(
            "Cannot select object fomr {}, it's a list",
            ctx.type_name(TypeRecord {
                definition_id: output.into(),
                wrapping: output_wrapping
            })
        ));
    }
    if target_wrapping.is_list() {
        return Err(format!("Cannot map object into {}, it's a list", target.display(ctx)));
    }
    if target_wrapping.is_required() && output_wrapping.is_nullable() {
        return Err(format!(
            "Cannot map nullable object {} into required one {}",
            ctx.type_name(TypeRecord {
                definition_id: output.into(),
                wrapping: output_wrapping
            }),
            target.display(ctx),
        ));
    }
    let fields = object
        .fields
        .into_iter()
        .map(|field| bind_selected_object_field(ctx, output, target, field))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BoundSelectedObjectValue { fields })
}

fn bind_selected_object_field<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    output: CompositeTypeId,
    parent_target: T,
    field: SelectedObjectField<'_>,
) -> Result<BoundSelectedObjectField<T::Id>, String> {
    let Some(target) = parent_target.field(ctx, field.key) else {
        return Err(format!(
            "Field '{}' does not exist on {}",
            field.key,
            parent_target.display(ctx)
        ));
    };
    let value = if let Some(value) = field.value {
        // The parent wrapping doesn't matter anymore, it was already handled.
        Some(bind_selected_value(ctx, (output, Wrapping::required()), target, value)?)
    } else {
        let field_ids = match output {
            CompositeTypeId::Interface(id) => ctx.graph[id].field_ids,
            CompositeTypeId::Object(id) => ctx.graph[id].field_ids,
            CompositeTypeId::Union(id) => {
                return Err(format!(
                    "Union {} does not have a field {}",
                    ctx[ctx.graph[id].name_id], field.key,
                ));
            }
        };
        let field_id = field_ids
            .into_iter()
            .find(|id| ctx[ctx.graph[*id].name_id] == field.key)
            .ok_or_else(|| {
                format!(
                    "Type {} does not have a field named '{}'",
                    ctx[ctx.definition_name_id(output.into())],
                    field.key
                )
            })?;
        ensure_type_compatibility(ctx, target, field_id)?;
        None
    };

    Ok(BoundSelectedObjectField {
        field: target.0.id(),
        value,
    })
}

fn bind_selected_list_value<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    (output, output_wrapping): (CompositeTypeId, Wrapping),
    (target, target_wrapping): (T, Wrapping),
    list: SelectedListValue<'_>,
) -> Result<BoundSelectedListValue<T::Id>, String> {
    if target_wrapping.is_required() && output_wrapping.is_nullable() {
        return Err(format!(
            "Cannot map nullable list {} into required one {}",
            ctx.type_name(TypeRecord {
                definition_id: output.into(),
                wrapping: output_wrapping
            }),
            target.display(ctx),
        ));
    }
    let Some(target_wrapping) = target_wrapping.without_list() else {
        return Err(format!("Cannot map a list into {}", target.display(ctx)));
    };
    let Some(output_wrapping) = output_wrapping.without_list() else {
        return Err(format!(
            "Cannot select a list in {}",
            ctx.type_name(TypeRecord {
                definition_id: output.into(),
                wrapping: output_wrapping
            })
        ));
    };
    let value = bind_selected_value(ctx, (output, output_wrapping), (target, target_wrapping), list.0)?;
    Ok(BoundSelectedListValue(value))
}

fn ensure_type_compatibility<T: Target>(
    ctx: &GraphBuilder<'_>,
    (target, wrapping): (T, Wrapping),
    field_id: FieldDefinitionId,
) -> Result<(), String> {
    let field = &ctx.graph[field_id];
    if !wrapping.is_equal_or_more_lenient_than(field.ty_record.wrapping) {
        return Err(format!(
            "Incompatible wrapping, cannot map {}.{} ({}) into {} ({})",
            ctx[ctx.definition_name_id(ctx.graph[field_id].parent_entity_id.into())],
            ctx[ctx.graph[field_id].name_id],
            ctx.type_name(ctx.graph[field_id].ty_record),
            target.display(ctx),
            ctx.type_name(TypeRecord {
                definition_id: target.type_definition(&ctx.graph),
                wrapping
            })
        ));
    }
    let is_compatible = match (
        target.type_definition(&ctx.graph),
        ctx.graph[field_id].ty_record.definition_id,
    ) {
        (TypeDefinitionId::Scalar(a), TypeDefinitionId::Scalar(b)) => a == b,
        (TypeDefinitionId::Enum(a), TypeDefinitionId::Enum(b)) => a == b,
        _ => false,
    };

    if !is_compatible {
        return Err(format!(
            "Cannot map {}.{} ({}) into {} ({})",
            ctx[ctx.definition_name_id(ctx.graph[field_id].parent_entity_id.into())],
            ctx[ctx.graph[field_id].name_id],
            ctx.type_name(ctx.graph[field_id].ty_record),
            target.display(ctx),
            ctx.type_name(TypeRecord {
                definition_id: target.type_definition(&ctx.graph),
                wrapping
            })
        ));
    }

    Ok(())
}
