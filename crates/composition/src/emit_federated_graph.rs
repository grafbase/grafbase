mod field_types_map;

use self::field_types_map::FieldTypesMap;
use crate::{
    composition_ir::{CompositionIr, FieldIr, KeyIr, StringsIr},
    subgraphs, Subgraphs, VecExt,
};
use grafbase_federated_graph as federated;
use itertools::Itertools;
use std::collections::BTreeSet;

struct Context<'a> {
    strings_ir: &'a mut StringsIr,
    field_types_map: &'a mut FieldTypesMap,
    out: &'a mut federated::FederatedGraph,
    subgraphs: &'a Subgraphs,
}

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(
    ir: CompositionIr,
    subgraphs: &Subgraphs,
) -> federated::FederatedGraph {
    let mut strings_ir = ir.strings;
    let mut field_types_map = FieldTypesMap::new(ir.definitions_by_name);
    let mut out = federated::FederatedGraph {
        enums: ir.enums,
        objects: ir.objects,
        interfaces: ir.interfaces,
        unions: ir.unions,
        scalars: ir.scalars,
        input_objects: ir.input_objects,
        ..Default::default()
    };

    let mut ctx = Context {
        strings_ir: &mut strings_ir,
        field_types_map: &mut field_types_map,
        out: &mut out,
        subgraphs,
    };

    emit_subgraphs(&mut ctx);
    emit_fields(&ir.fields, &mut ctx);

    emit_union_members(&ir.union_members, &mut ctx);
    emit_keys(&ir.resolvable_keys, &mut ctx);

    out.field_types = field_types_map.field_types;
    out.strings = strings_ir.strings;

    out
}

fn emit_fields(
    ir_fields: &[FieldIr],
    Context {
        strings_ir,
        field_types_map,
        out,
        subgraphs,
    }: &mut Context<'_>,
) {
    for FieldIr {
        parent_name,
        field_name,
        field_type,
        arguments,
    } in ir_fields
    {
        let field_type_id = field_types_map.insert(subgraphs.walk(*field_type));
        let field_name = strings_ir.insert(subgraphs.walk(*field_name));
        let arguments = arguments
            .iter()
            .map(|(arg_name, arg_type)| federated::FieldArgument {
                name: strings_ir.insert(subgraphs.walk(*arg_name)),
                type_id: field_types_map.insert(subgraphs.walk(*arg_type)),
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

        match field_types_map.definitions[parent_name] {
            federated::Definition::Object(object_id) => {
                let field_id = push_field(&mut out.fields);
                out.object_fields.push(federated::ObjectField {
                    object_id,
                    field_id,
                })
            }
            federated::Definition::Interface(interface_id) => {
                let field_id = push_field(&mut out.fields);
                out.interface_fields.push(federated::InterfaceField {
                    interface_id,
                    field_id,
                })
            }
            federated::Definition::InputObject(input_object_id) => {
                out[input_object_id]
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
    Context {
        field_types_map,
        out,
        ..
    }: &mut Context<'_>,
) {
    for (union_name, members) in &ir_members.iter().group_by(|(union_name, _)| union_name) {
        let federated::Definition::Union(union_id) = field_types_map.definitions[union_name] else {
            continue;
        };
        let union = &mut out[union_id];

        for (_, member) in members {
            let federated::Definition::Object(object_id) = field_types_map.definitions[member]
            else {
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
                            && ctx.strings_ir[field_name]
                                == ctx.subgraphs.walk(selection.field).as_str()
                    })
                    .map(|of| of.field_id)
                    .unwrap(),
                federated::Definition::Interface(interface_id) => ctx
                    .out
                    .interface_fields
                    .iter()
                    .find(|interface_field| {
                        interface_field.interface_id == interface_id
                            && ctx.strings_ir[ctx.out[interface_field.field_id].name]
                                == ctx.subgraphs.walk(selection.field).as_str()
                    })
                    .map(|of| of.field_id)
                    .unwrap(),
                _ => unreachable!(),
            };

            let field_ty = ctx.field_types_map[ctx.out[field].field_type_id].kind;
            federated::Selection {
                field,
                subselection: attach_selection(&selection.subselection, field_ty, ctx),
            }
        })
        .collect()
}

fn emit_subgraphs(ctx: &mut Context<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraphs() {
        ctx.out.subgraphs.push(federated::Subgraph {
            name: ctx.strings_ir.insert(subgraph.name()),
            url: ctx.strings_ir.insert(subgraph.url()),
        });
    }
}
