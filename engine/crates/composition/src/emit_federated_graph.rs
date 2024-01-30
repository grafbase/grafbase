mod context;
mod field_types_map;

use self::context::Context;
use crate::{
    composition_ir::{CompositionIr, FieldIr, InputValueDefinitionIr, KeyIr},
    subgraphs, Subgraphs, VecExt,
};
use federated::RootOperationTypes;
use graphql_federated_graph as federated;
use itertools::Itertools;
use std::{collections::BTreeSet, mem};

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(mut ir: CompositionIr, subgraphs: &Subgraphs) -> federated::FederatedGraph {
    let mut out = federated::FederatedGraphV2 {
        enums: mem::take(&mut ir.enums),
        enum_values: mem::take(&mut ir.enum_values),
        objects: mem::take(&mut ir.objects),
        interfaces: mem::take(&mut ir.interfaces),
        unions: mem::take(&mut ir.unions),
        scalars: mem::take(&mut ir.scalars),
        input_objects: mem::take(&mut ir.input_objects),
        directives: mem::take(&mut ir.directives),
        input_value_definitions: vec![],
        strings: Vec::new(),
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
    emit_interface_impls(&mut ctx);
    emit_fields(mem::take(&mut ir.fields), &mut ctx);
    emit_union_members(&ir.union_members, &mut ctx);
    emit_keys(&ir.keys, &mut ctx);
    emit_input_value_definitions(&ir.input_value_definitions, &mut ctx);
    push_object_fields_from_interface_entities(&ir.object_fields_from_entity_interfaces, &mut ctx);

    drop(ctx);

    federated::FederatedGraph::V2(out)
}

fn emit_input_value_definitions(input_value_definitions: &[InputValueDefinitionIr], ctx: &mut Context<'_>) {
    ctx.out.input_value_definitions = input_value_definitions
        .iter()
        .map(
            |InputValueDefinitionIr {
                 name,
                 r#type,
                 directives,
                 description,
             }| federated::InputValueDefinition {
                name: *name,
                field_type_id: ctx.insert_field_type(ctx.subgraphs.walk(*r#type)),
                directives: *directives,
                description: *description,
            },
        )
        .collect()
}

fn push_object_fields_from_interface_entities(
    object_fields_from_entity_interfaces: &BTreeSet<(federated::StringId, federated::FieldId)>,
    ctx: &mut Context<'_>,
) {
    for (object_name, field_id) in object_fields_from_entity_interfaces {
        let federated::Definition::Object(object_id) = ctx.definitions[object_name] else {
            continue;
        };
        ctx.push_object_field(object_id, *field_id);
    }
}

fn emit_interface_impls(ctx: &mut Context<'_>) {
    for (implementee, implementer) in ctx.subgraphs.iter_interface_impls() {
        let implementer = ctx.insert_string(ctx.subgraphs.walk(implementer));
        let implementee = ctx.insert_string(ctx.subgraphs.walk(implementee));

        let federated::Definition::Interface(implementee) = ctx.definitions[&implementee] else {
            continue;
        };

        match ctx.definitions[&implementer] {
            federated::Definition::Object(object_id) => {
                ctx.out.objects[object_id.0].implements_interfaces.push(implementee);
            }
            federated::Definition::Interface(interface_id) => {
                ctx.out.interfaces[interface_id.0]
                    .implements_interfaces
                    .push(implementee);
            }
            _ => unreachable!(),
        }
    }
}

fn emit_fields<'a>(ir_fields: Vec<FieldIr>, ctx: &mut Context<'a>) {
    // We have to accumulate the `@provides` and `@requires` and delay emitting them because
    // attach_selection() depends on all fields having been populated first.
    let mut field_provides: Vec<(
        federated::FieldId,
        federated::SubgraphId,
        federated::Definition,
        &'a [subgraphs::Selection],
    )> = Vec::new();
    let mut field_requires: Vec<(
        federated::FieldId,
        federated::SubgraphId,
        federated::Definition,
        &'a [subgraphs::Selection],
    )> = Vec::new();

    for FieldIr {
        parent_definition,
        field_name,
        field_type,
        arguments,
        resolvable_in,
        provides,
        requires,
        composed_directives,
        overrides,
        description,
    } in ir_fields
    {
        let r#type = ctx.insert_field_type(ctx.subgraphs.walk(field_type));
        let field_name = ctx.insert_string(ctx.subgraphs.walk(field_name));

        let push_field =
            |ctx: &mut Context<'a>, parent: federated::Definition, composed_directives: federated::Directives| {
                let field = federated::Field {
                    name: field_name,
                    field_type_id: r#type,
                    arguments,
                    overrides,

                    provides: Vec::new(),
                    requires: Vec::new(),
                    resolvable_in,
                    composed_directives,
                    description,
                };

                let id = federated::FieldId(ctx.out.fields.push_return_idx(field));

                for (subgraph_id, definition, provides) in provides.iter().filter_map(|field_id| {
                    let field = ctx.subgraphs.walk_field(*field_id);
                    field.directives().provides().map(|provides| {
                        let field_type_name = ctx.insert_string(field.r#type().type_name());
                        (
                            federated::SubgraphId(field.parent_definition().subgraph_id().idx()),
                            ctx.definitions[&field_type_name],
                            provides,
                        )
                    })
                }) {
                    field_provides.push((id, subgraph_id, definition, provides));
                }

                for (subgraph_id, provides) in requires.iter().filter_map(|field_id| {
                    let field = ctx.subgraphs.walk_field(*field_id);
                    field.directives().requires().map(|provides| {
                        (
                            federated::SubgraphId(field.parent_definition().subgraph_id().idx()),
                            provides,
                        )
                    })
                }) {
                    field_requires.push((id, subgraph_id, parent, provides));
                }

                id
            };

        match parent_definition {
            parent @ federated::Definition::Object(object_id) => {
                let field_id = push_field(ctx, parent, composed_directives);
                ctx.push_object_field(object_id, field_id);
            }
            parent @ federated::Definition::Interface(interface_id) => {
                let field_id = push_field(ctx, parent, composed_directives);
                ctx.push_interface_field(interface_id, field_id);
            }
            _ => unreachable!(),
        }
    }

    for (field_id, subgraph_id, definition, provides) in field_provides {
        let fields = attach_selection(provides, definition, ctx);
        ctx.out.fields[field_id.0]
            .provides
            .push(federated::FieldProvides { subgraph_id, fields });
    }

    for (field_id, subgraph_id, definition, requires) in field_requires {
        let fields = attach_selection(requires, definition, ctx);
        ctx.out.fields[field_id.0]
            .requires
            .push(federated::FieldRequires { subgraph_id, fields });
    }
}

fn emit_union_members(ir_members: &BTreeSet<(federated::StringId, federated::StringId)>, ctx: &mut Context<'_>) {
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
    for KeyIr {
        parent,
        key_id,
        is_interface_object,
        resolvable,
    } in keys
    {
        let key = ctx.subgraphs.walk(*key_id);
        let selection = attach_selection(key.fields(), *parent, ctx);
        let key = federated::Key {
            subgraph_id: federated::SubgraphId(key.parent_definition().subgraph_id().idx()),
            fields: selection,
            is_interface_object: *is_interface_object,
            resolvable: *resolvable,
        };

        match parent {
            federated::Definition::Object(object_id) => {
                ctx.out[*object_id].keys.push(key);
            }
            federated::Definition::Interface(interface_id) => {
                ctx.out[*interface_id].keys.push(key);
            }
            _ => unreachable!("non-object, non-interface key parent"),
        }
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
