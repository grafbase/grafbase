mod context;
mod field_types_map;

use self::context::Context;
use crate::{
    composition_ir::{CompositionIr, FieldIr, KeyIr},
    subgraphs, Subgraphs, VecExt,
};
use federated::RootOperationTypes;
use graphql_federated_graph as federated;
use itertools::Itertools;
use std::{collections::BTreeSet, mem};

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(mut ir: CompositionIr, subgraphs: &Subgraphs) -> federated::FederatedGraph {
    let mut out = federated::FederatedGraph {
        enums: mem::take(&mut ir.enums),
        objects: mem::take(&mut ir.objects),
        interfaces: mem::take(&mut ir.interfaces),
        unions: mem::take(&mut ir.unions),
        scalars: mem::take(&mut ir.scalars),
        input_objects: mem::take(&mut ir.input_objects),
        strings: mem::take(&mut ir.strings.strings),
        subgraphs: vec![],
        root_operation_types: RootOperationTypes {
            query: ir.query_type.unwrap(),
            mutation: ir.mutation_type,
            subscription: ir.subscription_type,
        },
        object_fields: vec![],
        interface_fields: vec![],
        fields: vec![],
        field_types: vec![],
    };

    let mut ctx = Context::new(&mut ir, subgraphs, &mut out);

    emit_subgraphs(&mut ctx);
    emit_fields(mem::take(&mut ir.fields), &mut ctx);

    emit_union_members(&ir.union_members, &mut ctx);
    emit_keys(&ir.resolvable_keys, &mut ctx);

    out
}

fn emit_fields(ir_fields: Vec<FieldIr>, ctx: &mut Context<'_>) {
    for FieldIr {
        parent_name,
        field_name,
        field_type,
        arguments,
        resolvable_in,
    } in ir_fields
    {
        let field_type_id = ctx.insert_field_type(ctx.subgraphs.walk(field_type));
        let field_name = ctx.insert_string(ctx.subgraphs.walk(field_name));
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
                resolvable_in,
                composed_directives: Vec::new(),
            };

            federated::FieldId(out.push_return_idx(field))
        };

        match ctx.definitions[&parent_name] {
            federated::Definition::Object(object_id) => {
                let field_id = push_field(&mut ctx.out.fields);
                ctx.push_object_field(object_id, field_id);
            }
            federated::Definition::Interface(interface_id) => {
                let field_id = push_field(&mut ctx.out.fields);
                ctx.push_interface_field(interface_id, field_id);
            }
            federated::Definition::InputObject(input_object_id) => {
                ctx.out[input_object_id].fields.push(federated::InputObjectField {
                    name: field_name,
                    field_type_id,
                });
            }
            _ => unreachable!(),
        }
    }
}

fn emit_union_members(ir_members: &BTreeSet<(subgraphs::StringId, subgraphs::StringId)>, ctx: &mut Context<'_>) {
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
) -> federated::FieldSet {
    selection_set
        .iter()
        .map(|selection| {
            let selection_field = ctx.insert_string(ctx.subgraphs.walk(selection.field));
            let field = ctx.selection_map[&(parent_id, selection_field)];
            let field_ty = ctx.out[ctx.out[field].field_type_id].kind;
            federated::FieldSetItem {
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
