mod context;
mod directive;
mod directive_definitions;
mod emit_extensions;
mod emit_fields;
mod federation_builtins;
mod field_types_map;

use self::{
    context::Context,
    directive::{
        emit_composite_spec_directive_definitions, emit_cost_directive_definition, emit_list_size_directive_definition,
        transform_arbitray_type_directives, transform_enum_value_directives, transform_field_directives,
        transform_input_value_directives, transform_type_directives,
    },
    directive_definitions::emit_directive_definitions,
    emit_extensions::*,
};
use crate::{
    Subgraphs, VecExt,
    composition_ir::{CompositionIr, FieldIr, InputValueDefinitionIr},
    federated_graph as federated, subgraphs,
};
use itertools::Itertools;
use std::collections::BTreeSet;

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(mut ir: CompositionIr, subgraphs: &Subgraphs) -> federated::FederatedGraph {
    let mut out = federated::FederatedGraph {
        enum_definitions: ir.enum_definitions.iter().map(|(ty, _)| ty.clone()).collect(),
        enum_values: ir.enum_values.iter().map(|v| v.federated.clone()).collect(),
        scalar_definitions: ir.scalar_definitions.iter().map(|(ty, _)| ty.clone()).collect(),
        objects: ir.objects.iter().map(|(obj, _directives)| obj.clone()).collect(),
        interfaces: ir.interfaces.iter().map(|(iface, _directives)| iface.clone()).collect(),
        unions: ir.unions.iter().map(|u| u.federated.clone()).collect(),
        input_objects: ir.input_objects.iter().map(|io| io.federated.clone()).collect(),
        subgraphs: vec![],
        directive_definitions: vec![],
        directive_definition_arguments: vec![],
        fields: vec![],
        input_value_definitions: vec![],
        strings: vec![],
        extensions: Vec::new(),
    };

    let mut ctx = Context::new(&mut ir, subgraphs, &mut out);

    // The fields of a given object or interface may not be contiguous in the IR because of entity interfaces.
    // Also, sort them by name.
    ir.fields
        .sort_unstable_by_key(|field| (field.parent_definition_name, &ctx[field.field_name]));

    let join_graph_enum_id = emit_subgraphs(&mut ctx);
    emit_extensions(&mut ctx, &ir);
    emit_input_value_definitions(&ir.input_value_definitions, &mut ctx);
    emit_fields(&ir.fields, &mut ctx);
    emit_directive_definitions(&ir, &mut ctx);

    emit_union_members_after_objects(&ir.union_members, &mut ctx);
    federation_builtins::emit_federation_builtins(&mut ctx, join_graph_enum_id);

    emit_directives_and_implements_interface(&mut ctx, ir);

    ctx.out.enum_values.sort_unstable_by_key(|v| v.enum_id);

    drop(ctx);

    out
}

fn emit_directives_and_implements_interface(ctx: &mut Context<'_>, mut ir: CompositionIr) {
    for (i, (_object, directives)) in ir.objects.iter().enumerate() {
        ctx.out[federated::ObjectId::from(i)].directives = transform_type_directives(
            ctx,
            federated::Definition::Object(federated::ObjectId::from(i)),
            directives,
        );
    }

    for (i, (_interface, directives)) in ir.interfaces.iter().enumerate() {
        ctx.out[federated::InterfaceId::from(i)].directives = transform_type_directives(
            ctx,
            federated::Definition::Interface(federated::InterfaceId::from(i)),
            directives,
        );
    }

    for (i, union) in ir.unions.iter().enumerate() {
        ctx.out.unions[i].directives = transform_type_directives(
            ctx,
            federated::Definition::Union(federated::UnionId::from(i)),
            &union.directives,
        );
    }

    for (i, input_object) in ir.input_objects.into_iter().enumerate() {
        ctx.out.input_objects[i].directives = transform_type_directives(
            ctx,
            federated::Definition::InputObject(federated::InputObjectId::from(i)),
            &input_object.directives,
        );
    }

    for (i, field) in ir.fields.into_iter().enumerate() {
        ctx.out.fields[i].directives = transform_field_directives(ctx, federated::FieldId::from(i), &field.directives);
    }

    for (i, enum_value) in ir.enum_values.into_iter().enumerate() {
        ctx.out.enum_values[i].directives = transform_enum_value_directives(ctx, &enum_value.directives);
    }

    for (i, input_value_definition) in ir.input_value_definitions.into_iter().enumerate() {
        ctx.out.input_value_definitions[i].directives =
            transform_input_value_directives(ctx, &input_value_definition.directives);
    }

    for (i, (_enum_definition, directives)) in ir.enum_definitions.iter_mut().enumerate() {
        if !directives.is_empty() {
            ctx.out.enum_definitions[i].directives = transform_arbitray_type_directives(ctx, directives);
        }
    }

    for (i, (_scalar_definition, directives)) in ir.scalar_definitions.iter_mut().enumerate() {
        if !directives.is_empty() {
            ctx.out.scalar_definitions[i].directives = transform_arbitray_type_directives(ctx, directives);
        }
    }

    emit_cost_directive_definition(ctx);
    emit_list_size_directive_definition(ctx);
    emit_composite_spec_directive_definitions(ctx);
    emit_interface_after_directives(ctx);
}

fn emit_input_value_definitions(input_value_definitions: &[InputValueDefinitionIr], ctx: &mut Context<'_>) {
    ctx.out.input_value_definitions = input_value_definitions
        .iter()
        .map(
            |InputValueDefinitionIr {
                 name,
                 r#type,
                 description,
                 default,
                 ..
             }| {
                let r#type = ctx.insert_field_type(ctx.subgraphs.walk(*r#type));
                let default = default
                    .as_ref()
                    .map(|default| ctx.insert_value_with_type(default, r#type.definition.as_enum()));

                federated::InputValueDefinition {
                    name: *name,
                    r#type,
                    directives: Vec::new(),
                    description: *description,
                    default,
                }
            },
        )
        .collect()
}

fn emit_interface_after_directives(ctx: &mut Context<'_>) {
    for (implementee_name, implementer_name) in ctx.subgraphs.iter_interface_impls() {
        let implementer = ctx.insert_string(ctx.subgraphs.walk(implementer_name));
        let implementee = ctx.insert_string(ctx.subgraphs.walk(implementee_name));

        let federated::Definition::Interface(implementee) = ctx.definitions[&implementee] else {
            continue;
        };

        match ctx.definitions[&implementer] {
            federated::Definition::Object(object_id) => {
                let object = &mut ctx.out.objects[usize::from(object_id)];
                object.implements_interfaces.push(implementee);

                for subgraph_id in ctx
                    .subgraphs
                    .subgraphs_implementing_interface(implementee_name, implementer_name)
                {
                    object.directives.push(federated::Directive::JoinImplements(
                        federated::JoinImplementsDirective {
                            subgraph_id: federated::SubgraphId::from(subgraph_id.idx()),
                            interface_id: implementee,
                        },
                    ));
                }
            }
            federated::Definition::Interface(interface_id) => {
                let interface = &mut ctx.out.interfaces[usize::from(interface_id)];
                interface.implements_interfaces.push(implementee);

                for subgraph_id in ctx
                    .subgraphs
                    .subgraphs_implementing_interface(implementee_name, implementer_name)
                {
                    interface.directives.push(federated::Directive::JoinImplements(
                        federated::JoinImplementsDirective {
                            subgraph_id: federated::SubgraphId::from(subgraph_id.idx()),
                            interface_id: implementee,
                        },
                    ));
                }
            }
            _ => unreachable!(),
        }
    }
}

fn emit_fields(fields: &[FieldIr], ctx: &mut Context<'_>) {
    emit_fields::for_each_field_group(fields, |parent_definition_name, fields| {
        let parent_definition_id = ctx.definitions[&parent_definition_name];
        let parent_entity_id = parent_definition_id
            .as_entity()
            .expect("Only interfaces & objects can have fields.");

        let start = ctx.out.fields.len();

        let mut fields = fields.to_owned();

        // Sort the fields by name.
        fields.sort_by(|a, b| ctx[a.field_name].cmp(&ctx[b.field_name]));

        for FieldIr {
            field_name,
            field_type,
            arguments,
            description,
            ..
        } in fields
        {
            let r#type = ctx.insert_field_type(ctx.subgraphs.walk(field_type));
            let name = ctx.insert_string(ctx.subgraphs.walk(field_name));
            let field = federated::Field {
                name,
                r#type,
                arguments,
                parent_entity_id,
                description,
                directives: Vec::new(),
            };

            let field_id = federated::FieldId::from(ctx.out.fields.push_return_idx(field));

            let selection_map_key = (parent_definition_id, field_name);
            ctx.selection_map.insert(selection_map_key, field_id);
        }

        let fields = federated::Fields {
            start: federated::FieldId::from(start),
            end: federated::FieldId::from(ctx.out.fields.len()),
        };

        match parent_entity_id {
            federated::EntityDefinitionId::Object(id) => {
                ctx.out.objects[usize::from(id)].fields = fields;
            }
            federated::EntityDefinitionId::Interface(id) => {
                ctx.out.interfaces[usize::from(id)].fields = fields;
            }
        }
    });
}

fn emit_union_members_after_objects(
    ir_members: &BTreeSet<(federated::StringId, federated::StringId)>,
    ctx: &mut Context<'_>,
) {
    for (union_name, members) in &ir_members.iter().chunk_by(|(union_name, _)| union_name) {
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

/// Attach a selection set defined in strings to a FederatedGraph, transforming the strings into
/// field ids.
fn attach_selection(
    selection_set: &[subgraphs::Selection],
    target: federated::Definition,
    ctx: &mut Context<'_>,
) -> federated::SelectionSet {
    selection_set
        .iter()
        .map(|selection| {
            match selection {
                subgraphs::Selection::Field(subgraphs::FieldSelection {
                    field,
                    arguments,
                    subselection,
                    has_directives: _,
                }) => {
                    if &ctx[*field] == "__typename" {
                        federated::Selection::Typename
                    } else {
                        let field_id = ctx.selection_map[&(target, *field)];
                        let field_ty = ctx.out[field_id].r#type.definition;
                        let field_arguments = ctx.out[field_id].arguments;
                        let (field_arguments_start, _) = field_arguments;
                        let field_arguments_start = usize::from(field_arguments_start);
                        let arguments = arguments
                            .iter()
                            .map(|(name, value)| {
                                // Here we assume the arguments are validated previously.
                                let arg_name = ctx.insert_string(ctx.subgraphs.walk(*name));
                                let argument = ctx.out[field_arguments]
                                    .iter()
                                    .position(|arg| arg.name == arg_name)
                                    .map(|idx| federated::InputValueDefinitionId::from(field_arguments_start + idx))
                                    .unwrap();

                                let argument_enum_type = ctx.out[argument].r#type.definition.as_enum();
                                let value = ctx.insert_value_with_type(value, argument_enum_type);

                                (argument, value)
                            })
                            .collect();

                        federated::Selection::Field(federated::FieldSelection {
                            field_id,
                            arguments,
                            subselection: attach_selection(subselection, field_ty, ctx),
                        })
                    }
                }
                subgraphs::Selection::InlineFragment {
                    on,
                    subselection,
                    has_directives: _,
                } => {
                    let on = ctx.insert_string(ctx.subgraphs.walk(*on));
                    let on = ctx.definitions[&on];

                    federated::Selection::InlineFragment {
                        on,
                        subselection: attach_selection(subselection, on, ctx),
                    }
                }
            }
        })
        .collect()
}

fn emit_subgraphs(ctx: &mut Context<'_>) -> federated::EnumDefinitionId {
    let join_namespace = ctx.insert_str("join");
    let join_graph_name = ctx.insert_str("Graph");
    let join_graph_enum_id = ctx.out.push_enum_definition(federated::EnumDefinitionRecord {
        namespace: Some(join_namespace),
        name: join_graph_name,
        directives: Vec::new(),
        description: None,
    });

    for subgraph in ctx.subgraphs.iter_subgraphs() {
        let name = ctx.insert_string(subgraph.name());
        let url = subgraph.url().map(|url| ctx.insert_string(url));
        let join_graph_enum_value_name = ctx.insert_str(&join_graph_enum_variant_name(subgraph.name().as_str()));
        let join_graph_enum_value_id = ctx.out.push_enum_value(federated::EnumValueRecord {
            enum_id: join_graph_enum_id,
            value: join_graph_enum_value_name,
            directives: vec![federated::Directive::JoinGraph(federated::JoinGraphDirective {
                name,
                url,
            })],
            description: None,
        });

        ctx.out.subgraphs.push(federated::Subgraph {
            name,
            join_graph_enum_value: join_graph_enum_value_id,
            url,
        });
    }

    join_graph_enum_id
}

fn join_graph_enum_variant_name(original_name: &str) -> String {
    let mut out = String::with_capacity(original_name.len());
    for char in original_name.chars() {
        match char {
            '-' | '_' | ' ' => out.push('_'),
            other => {
                for char in other.to_uppercase() {
                    out.push(char);
                }
            }
        }
    }
    out
}
