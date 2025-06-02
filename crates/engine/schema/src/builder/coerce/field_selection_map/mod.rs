mod model;
mod target;

pub(crate) use model::*;
use std::collections::VecDeque;
use target::*;

use ::field_selection_map::*;
use wrapping::Wrapping;

use crate::{
    CompositeTypeId, EntityDefinitionId, FieldDefinitionId, InputValueDefinitionId, StringId, SubgraphId,
    TypeDefinitionId, TypeRecord, builder::GraphBuilder,
};

use super::field_set::is_disjoint;

impl GraphBuilder<'_> {
    pub(crate) fn parse_field_selection_map_for_argument(
        &mut self,
        source: TypeRecord,
        target_field_id: FieldDefinitionId,
        target_argument_id: InputValueDefinitionId,
        field_selection_map: &str,
    ) -> Result<BoundSelectedValue<InputValueDefinitionId>, String> {
        let selected_value = SelectedValue::try_from(field_selection_map).map_err(|err| format!("\n{err}\n"))?;
        let wrapping = self.graph[target_argument_id].ty_record.wrapping;
        bind_selected_value(
            self,
            source,
            (
                InputTarget::Argument {
                    field_id: target_field_id,
                    argument_id: target_argument_id,
                },
                wrapping,
            ),
            selected_value,
        )
    }

    pub(crate) fn parse_field_selection_map_for_derived_field(
        &mut self,
        source_id: EntityDefinitionId,
        subgraph_id: SubgraphId,
        target_field_id: FieldDefinitionId,
        field_selection_map: &str,
    ) -> Result<BoundSelectedValue<FieldDefinitionId>, String> {
        let selected_value = SelectedValue::try_from(field_selection_map).map_err(|err| format!("\n{err}\n"))?;
        let wrapping = self.graph[target_field_id].ty_record.wrapping;
        bind_selected_value(
            self,
            TypeRecord {
                definition_id: source_id.into(),
                wrapping: Wrapping::default().non_null(),
            },
            ((subgraph_id, target_field_id), wrapping),
            selected_value,
        )
    }
}

fn bind_selected_value<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    source: TypeRecord,
    target: (T, Wrapping),
    selected_value: SelectedValue<'_>,
) -> Result<BoundSelectedValue<T::Id>, String> {
    let alternatives = selected_value
        .alternatives
        .into_iter()
        .map(|entry| bind_selected_value_entry(ctx, source, target, entry))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BoundSelectedValue { alternatives })
}

fn bind_selected_value_entry<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    source: TypeRecord,
    target: (T, Wrapping),
    selected_value: SelectedValueEntry<'_>,
) -> Result<BoundSelectedValueEntry<T::Id>, String> {
    match selected_value {
        SelectedValueEntry::Path(path) => {
            let (path, _) = bind_path(ctx, source, path)?;
            let last = *path.0.last().unwrap();
            ensure_type_compatibility(
                ctx,
                FieldDisplay { ctx, field_id: last },
                ctx.graph[last].ty_record,
                target,
            )?;
            Ok(BoundSelectedValueEntry::Path(path))
        }
        SelectedValueEntry::Object { path, object } => {
            if let Some(path) = path {
                let (path, source) = bind_path(ctx, source, path)?;
                let object = bind_selected_object_value(ctx, source, target, object)?;
                Ok(BoundSelectedValueEntry::Object {
                    path: Some(path),
                    object,
                })
            } else {
                let object = bind_selected_object_value(ctx, source, target, object)?;
                Ok(BoundSelectedValueEntry::Object { path: None, object })
            }
        }
        SelectedValueEntry::List { path, list } => {
            if let Some(path) = path {
                let (path, source) = bind_path(ctx, source, path)?;
                let list = bind_selected_list_value(ctx, source, target, list)?;
                Ok(BoundSelectedValueEntry::List { path: Some(path), list })
            } else {
                let list = bind_selected_list_value(ctx, source, target, list)?;
                Ok(BoundSelectedValueEntry::List { path: None, list })
            }
        }
        SelectedValueEntry::Identity => {
            ensure_type_compatibility(ctx, ctx.type_name(source), source, target)?;
            Ok(BoundSelectedValueEntry::Identity)
        }
    }
}

fn bind_path(
    ctx: &mut GraphBuilder<'_>,
    source: TypeRecord,
    path: Path<'_>,
) -> Result<(BoundPath, TypeRecord), String> {
    if source.wrapping.is_list() {
        return Err(format!(
            "Cannot select a field from {}, it's a list",
            ctx.type_name(source)
        ));
    }
    let mut wrapping = source.wrapping;

    let Some(definition_id) = source.definition_id.as_composite_type() else {
        return Err(format!("Type {} does not have any fields", ctx.type_name(source)));
    };
    let mut definition_id: TypeDefinitionId = if let Some(ty) = path.ty {
        bind_type_condition(ctx, definition_id, ty)?.into()
    } else {
        definition_id.into()
    };

    let mut out = Vec::new();
    let mut segments = VecDeque::from(path.segments);
    while let Some(segment) = segments.pop_front() {
        let field_ids = match definition_id {
            TypeDefinitionId::Interface(id) => ctx.graph[id].field_ids,
            TypeDefinitionId::Object(id) => ctx.graph[id].field_ids,
            _ => {
                return Err(format!(
                    "Type {} does not have any fields",
                    ctx[ctx.definition_name_id(definition_id)]
                ));
            }
        };
        let field_id = field_ids
            .into_iter()
            .find(|id| ctx[ctx.graph[*id].name_id] == segment.field)
            .ok_or_else(|| {
                format!(
                    "Type {} does not have a field named '{}'",
                    ctx[ctx.definition_name_id(definition_id)],
                    segment.field
                )
            })?;
        out.push(field_id);
        definition_id = ctx.graph[field_id].ty_record.definition_id;
        let parent_source_is_nullable = wrapping.is_nullable();
        wrapping = ctx.graph[field_id].ty_record.wrapping;
        if parent_source_is_nullable {
            wrapping = wrapping.without_non_null();
        }

        if let Some(ty) = segment.ty {
            if let Some(id) = definition_id.as_composite_type() {
                definition_id = bind_type_condition(ctx, id, ty)?.into();
            } else {
                return Err(format!(
                    "Field '{}' doesn't return an object, interface or union and thus cannot have type condition '{}'",
                    ctx[ctx.graph[field_id].name_id], ty
                ));
            }
        }
    }

    Ok((
        BoundPath(out),
        TypeRecord {
            definition_id,
            wrapping,
        },
    ))
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
    source: TypeRecord,
    (target, target_wrapping): (T, Wrapping),
    object: SelectedObjectValue<'_>,
) -> Result<BoundSelectedObjectValue<T::Id>, String> {
    if source.wrapping.is_list() {
        return Err(format!(
            "Cannot select object from {}, it's a list",
            ctx.type_name(source)
        ));
    }
    if target_wrapping.is_list() {
        return Err(format!("Cannot map object into {}, it's a list", target.display(ctx)));
    }
    if target_wrapping.is_required() && source.wrapping.is_nullable() {
        return Err(format!(
            "Cannot map nullable object {} into required one {}",
            ctx.type_name(source),
            target.display(ctx),
        ));
    }
    let mut nested_target_fields = target.fields(ctx);
    if nested_target_fields.is_empty() {
        return Err(format!(
            "Cannot map object into {}, it's not an object nor an interface",
            target.display(ctx)
        ));
    }
    let mut fields = object
        .fields
        .into_iter()
        .map(|field| bind_selected_object_field(ctx, source.definition_id, target, &mut nested_target_fields, field))
        .collect::<Result<Vec<_>, _>>()?;
    for (name_id, (target_field, wrapping)) in nested_target_fields {
        let id = target_field.id();
        if fields.iter().any(|field| field.id == id) {
            continue;
        }
        if let Some(default_value) = target_field.default_value(ctx) {
            fields.push(BoundSelectedObjectField {
                id,
                value: SelectedValueOrField::DefaultValue(default_value),
            });
            continue;
        } else if wrapping.is_required() {
            return Err(format!(
                "For {}, field '{}' is required but it's missing from the FieldSelectionMap",
                target.display(ctx),
                ctx[name_id]
            ));
        }
    }
    Ok(BoundSelectedObjectValue { fields })
}

fn bind_selected_object_field<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    source: TypeDefinitionId,
    parent_target: T,
    target_fields: &mut Vec<(StringId, (T, Wrapping))>,
    field: SelectedObjectField<'_>,
) -> Result<BoundSelectedObjectField<T::Id>, String> {
    let Some(ix) = target_fields.iter().position(|(name_id, _)| ctx[*name_id] == field.key) else {
        return Err(format!(
            "Field '{}' does not exist on {}",
            field.key,
            parent_target.display(ctx)
        ));
    };
    let (_, target) = target_fields.swap_remove(ix);

    let value = if let Some(value) = field.value {
        // The parent wrapping doesn't matter anymore, it was already handled.
        SelectedValueOrField::Value(bind_selected_value(
            ctx,
            TypeRecord {
                definition_id: source,
                wrapping: Wrapping::default().non_null(),
            },
            target,
            value,
        )?)
    } else {
        let field_ids = match source.as_composite_type() {
            Some(CompositeTypeId::Interface(id)) => ctx.graph[id].field_ids,
            Some(CompositeTypeId::Object(id)) => ctx.graph[id].field_ids,
            Some(CompositeTypeId::Union(id)) => {
                return Err(format!(
                    "Union {} does not have a field {}",
                    ctx[ctx.graph[id].name_id], field.key,
                ));
            }
            None => {
                return Err(format!(
                    "Type {} does not have any fields",
                    ctx[ctx.definition_name_id(source)]
                ));
            }
        };
        let field_id = field_ids
            .into_iter()
            .find(|id| ctx[ctx.graph[*id].name_id] == field.key)
            .ok_or_else(|| {
                format!(
                    "Type {} does not have a field named '{}'",
                    ctx[ctx.definition_name_id(source)],
                    field.key
                )
            })?;
        ensure_type_compatibility(
            ctx,
            FieldDisplay { ctx, field_id },
            ctx.graph[field_id].ty_record,
            target,
        )?;
        SelectedValueOrField::Field(field_id)
    };

    Ok(BoundSelectedObjectField {
        id: target.0.id(),
        value,
    })
}

fn bind_selected_list_value<T: Target>(
    ctx: &mut GraphBuilder<'_>,
    source: TypeRecord,
    (target, target_wrapping): (T, Wrapping),
    list: SelectedListValue<'_>,
) -> Result<BoundSelectedListValue<T::Id>, String> {
    if target_wrapping.is_required() && source.wrapping.is_nullable() {
        return Err(format!(
            "Cannot map nullable list {} into required one {}",
            ctx.type_name(source),
            target.display(ctx),
        ));
    }
    let Some(target_wrapping) = target_wrapping.without_list() else {
        return Err(format!("Cannot map a list into {}", target.display(ctx)));
    };
    let Some(wrapping) = source.wrapping.without_list() else {
        return Err(format!("Cannot select a list in {}", ctx.type_name(source)));
    };
    let value = bind_selected_value(
        ctx,
        TypeRecord {
            definition_id: source.definition_id,
            wrapping,
        },
        (target, target_wrapping),
        list.0,
    )?;
    Ok(BoundSelectedListValue(value))
}

fn ensure_type_compatibility<T: Target>(
    ctx: &GraphBuilder<'_>,
    source_display: impl std::fmt::Display,
    source: TypeRecord,
    (target, wrapping): (T, Wrapping),
) -> Result<(), String> {
    if !wrapping.is_equal_or_more_lenient_than(source.wrapping) {
        return Err(format!(
            "Incompatible wrapping, cannot map {} into {} ({})",
            source_display,
            target.display(ctx),
            ctx.type_name(TypeRecord {
                definition_id: target.type_definition(&ctx.graph),
                wrapping
            })
        ));
    }
    let is_compatible = match (target.type_definition(&ctx.graph), source.definition_id) {
        (TypeDefinitionId::Scalar(a), TypeDefinitionId::Scalar(b)) => a == b,
        (TypeDefinitionId::Enum(a), TypeDefinitionId::Enum(b)) => a == b,
        (_, TypeDefinitionId::Object(_) | TypeDefinitionId::Interface(_)) => {
            return Err(format!(
                "Fields must be explictely selected on {}, it's not a scalar or enum",
                source_display,
            ));
        }
        _ => false,
    };

    if !is_compatible {
        return Err(format!(
            "Cannot map {} into {} ({})",
            source_display,
            target.display(ctx),
            ctx.type_name(TypeRecord {
                definition_id: target.type_definition(&ctx.graph),
                wrapping
            })
        ));
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct FieldDisplay<'a> {
    ctx: &'a GraphBuilder<'a>,
    field_id: FieldDefinitionId,
}

impl std::fmt::Display for FieldDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { ctx, field_id } = *self;
        let field = &ctx.graph[field_id];
        write!(
            f,
            "{}.{} ({})",
            ctx[ctx.definition_name_id(field.parent_entity_id.into())],
            ctx[field.name_id],
            ctx.type_name(field.ty_record),
        )
    }
}
