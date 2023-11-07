mod context;
mod field_types_map;

use self::{context::Context, field_types_map::FieldTypesMap};
use crate::{
    composition_ir::{CompositionIr, FieldIr, KeyIr},
    subgraphs, Subgraphs, VecExt,
};
use grafbase_federated_graph as federated;
use itertools::Itertools;
use std::collections::BTreeSet;

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(
    ir: CompositionIr,
    subgraphs: &Subgraphs,
) -> federated::FederatedGraph {
    let mut field_types_map = FieldTypesMap::default();
    let mut out = federated::FederatedGraph {
        enums: ir.enums,
        objects: ir.objects,
        interfaces: ir.interfaces,
        unions: ir.unions,
        scalars: ir.scalars,
        input_objects: ir.input_objects,
        strings: ir.strings.strings,
        ..Default::default()
    };

    let mut ctx = Context {
        definitions: ir.definitions_by_name,
        strings_map: ir.strings.map,
        field_types_map: &mut field_types_map,
        out: &mut out,
        subgraphs,
    };

    emit_subgraphs(&mut ctx);
    emit_fields(&ir.fields, &mut ctx);

    emit_union_members(&ir.union_members, &mut ctx);
    emit_keys(&ir.resolvable_keys, &mut ctx);

    out
}

fn emit_fields(ir_fields: &[FieldIr], ctx: &mut Context<'_>) {
    for FieldIr {
        parent_name,
        field_name,
        field_type,
        arguments,
    } in ir_fields
    {
        let field_type_id = ctx.insert_field_type(ctx.subgraphs.walk(*field_type));
        let field_name = ctx.insert_string(ctx.subgraphs.walk(*field_name));
        let arguments = arguments
            .iter()
            .map(|(arg_name, arg_type)| federated::FieldArgument {
                name: ctx.insert_string(ctx.subgraphs.walk(*arg_name)),
                type_id: ctx.insert_field_type(ctx.subgraphs.walk(*arg_type)),
            })
            .collect();

        let push_field = |out: &mut Vec<_>| {
            let field = federated::Field {
                name: field_name,
                field_type_id,
                arguments,

                provides: Vec::new(),
                requires: Vec::new(),
                resolvable_in: Vec::new(),
                composed_directives: Vec::new(),
            };

            federated::FieldId(out.push_return_idx(field))
        };

        match ctx.definitions[parent_name] {
            federated::Definition::Object(object_id) => {
                let field_id = push_field(&mut ctx.out.fields);
                ctx.out.object_fields.push(federated::ObjectField {
                    object_id,
                    field_id,
                })
            }
            federated::Definition::Interface(interface_id) => {
                let field_id = push_field(&mut ctx.out.fields);
                ctx.out.interface_fields.push(federated::InterfaceField {
                    interface_id,
                    field_id,
                })
            }
            federated::Definition::InputObject(input_object_id) => {
                ctx.out[input_object_id]
                    .fields
                    .push(federated::InputObjectField {
                        name: field_name,
                        field_type_id,
                    });
            }
            _ => unreachable!(),
        }
    }
}

fn emit_union_members(
    ir_members: &BTreeSet<(subgraphs::StringId, subgraphs::StringId)>,
    ctx: &mut Context<'_>,
) {
    for (union_name, members) in &ir_members.iter().group_by(|(union_name, _)| union_name) {
        let federated::Definition::Union(union_id) = ctx.definitions[union_name] else {
            continue;
        };
        let union = &mut ctx.out[union_id];

        for (_, member) in members {
            let federated::Definition::Object(object_id) = ctx.definitions[member] else {
                continue;
            };
            union.members.push(object_id);
        }
    }
}

fn emit_keys(keys: &[KeyIr], ctx: &mut Context<'_>) {
    for KeyIr { object_id, key_id } in keys {
        let parent_id = federated::Definition::Object(*object_id);
        let key = ctx.subgraphs.walk(*key_id);
        let selection = attach_selection(key.fields(), parent_id, ctx);
        ctx.out[*object_id].resolvable_keys.push(federated::Key {
            subgraph_id: federated::SubgraphId(key.parent_definition().subgraph().id.idx()),
            fields: selection,
        });
    }
}

/// Attach a selection set defined in strings to a FederatedGraph, transforming the strings into
/// field ids.
fn attach_selection(
    selection_set: &[subgraphs::Selection],
    parent_id: federated::Definition,
    ctx: &mut Context<'_>,
) -> federated::SelectionSet {
    selection_set
        .iter()
        .map(|selection| {
            let field = match parent_id {
                federated::Definition::Object(object_id) => ctx
                    .out
                    .object_fields
                    .iter()
                    .find(|object_field| {
                        let field_name = ctx.out[object_field.field_id].name;
                        object_field.object_id == object_id
                            && ctx.out[field_name] == ctx.subgraphs.walk(selection.field).as_str()
                    })
                    .map(|of| of.field_id)
                    .unwrap(),
                federated::Definition::Interface(interface_id) => ctx
                    .out
                    .interface_fields
                    .iter()
                    .find(|interface_field| {
                        interface_field.interface_id == interface_id
                            && ctx.out[ctx.out[interface_field.field_id].name]
                                == ctx.subgraphs.walk(selection.field).as_str()
                    })
                    .map(|of| of.field_id)
                    .unwrap(),
                _ => unreachable!(),
            };

            let field_ty = ctx.out[ctx.out[field].field_type_id].kind;
            federated::Selection {
                field,
                subselection: attach_selection(&selection.subselection, field_ty, ctx),
            }
        })
        .collect()
}

fn emit_subgraphs(ctx: &mut Context<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraphs() {
        let name = ctx.insert_string(subgraph.name());
        let url = ctx.insert_string(subgraph.url());
        ctx.out.subgraphs.push(federated::Subgraph { name, url });
    }
}
