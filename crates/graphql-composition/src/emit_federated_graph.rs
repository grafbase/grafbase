mod attach_argument_selection;
mod context;
mod directive;
mod emit_fields;
mod field_types_map;

use self::context::Context;
use crate::{
    composition_ir::{CompositionIr, FieldIr, InputValueDefinitionIr},
    subgraphs::{self},
    Subgraphs, VecExt,
};
use directive::{
    transform_arbitray_type_directives, transform_enum_value_directives, transform_field_directives,
    transform_input_value_directives, transform_type_directives,
};
use graphql_federated_graph::{self as federated};
use itertools::Itertools;
use std::{collections::BTreeSet, mem};

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(mut ir: CompositionIr, subgraphs: &Subgraphs) -> federated::FederatedGraph {
    let mut out = federated::FederatedGraph {
        type_definitions: ir.type_definitions.iter().map(|ty| ty.federated.clone()).collect(),
        enum_values: ir.enum_values.iter().map(|v| v.federated.clone()).collect(),
        objects: ir.objects.clone(),
        interfaces: ir.interfaces.clone(),
        unions: ir.unions.iter().map(|u| u.federated.clone()).collect(),
        input_objects: ir.input_objects.iter().map(|io| io.federated.clone()).collect(),
        root_operation_types: federated::RootOperationTypes {
            query: ir.query_type.unwrap(),
            mutation: ir.mutation_type,
            subscription: ir.subscription_type,
        },
        subgraphs: vec![],
        fields: vec![],
        input_value_definitions: vec![],
        strings: vec![],
    };

    let mut ctx = Context::new(&mut ir, subgraphs, &mut out);

    // The fields of a given object or interface may not be contiguous in the IR because of entity interfaces.
    // Also, sort them by name.
    ir.fields
        .sort_unstable_by_key(|field| (field.parent_definition_name, &ctx[field.field_name]));

    emit_subgraphs(&mut ctx);
    emit_input_value_definitions(&ir.input_value_definitions, &mut ctx);
    emit_fields(&ir.fields, &mut ctx);

    emit_union_members_after_objects(&ir.union_members, &mut ctx);

    emit_directives_and_implements_interface(ctx, ir);

    out.enum_values.sort_unstable_by_key(|v| v.enum_id);

    out
}

fn emit_directives_and_implements_interface(mut ctx: Context<'_>, mut ir: CompositionIr) {
    for (i, object) in ir.objects.into_iter().enumerate() {
        let type_def = &mut ir.type_definitions[usize::from(object.type_definition_id)];
        ctx.out[object.type_definition_id].directives = transform_type_directives(
            &mut ctx,
            federated::Definition::Object(federated::ObjectId::from(i)),
            mem::take(&mut type_def.directives),
        );
    }

    for (i, interface) in ir.interfaces.into_iter().enumerate() {
        let type_def = &mut ir.type_definitions[usize::from(interface.type_definition_id)];
        ctx.out[interface.type_definition_id].directives = transform_type_directives(
            &mut ctx,
            federated::Definition::Interface(federated::InterfaceId::from(i)),
            mem::take(&mut type_def.directives),
        );
    }

    for (i, union) in ir.unions.into_iter().enumerate() {
        ctx.out.unions[i].directives = transform_type_directives(
            &mut ctx,
            federated::Definition::Union(federated::UnionId::from(i)),
            union.directives,
        );
    }

    for (i, input_object) in ir.input_objects.into_iter().enumerate() {
        ctx.out.input_objects[i].directives = transform_type_directives(
            &mut ctx,
            federated::Definition::InputObject(federated::InputObjectId::from(i)),
            input_object.directives,
        );
    }

    for (i, field) in ir.fields.into_iter().enumerate() {
        ctx.out.fields[i].directives =
            transform_field_directives(&mut ctx, federated::FieldId::from(i), field.directives);
    }

    for (i, enum_value) in ir.enum_values.into_iter().enumerate() {
        ctx.out.enum_values[i].directives = transform_enum_value_directives(&mut ctx, enum_value.directives);
    }

    for (i, input_value_definition) in ir.input_value_definitions.into_iter().enumerate() {
        ctx.out.input_value_definitions[i].directives =
            transform_input_value_directives(&mut ctx, input_value_definition.directives);
    }

    // Anything left from this point is treated with the default transformation. Should only be
    // enums and scalars today.
    for (i, type_def) in ir.type_definitions.iter_mut().enumerate() {
        if !type_def.directives.is_empty() {
            ctx.out.type_definitions[i].directives =
                transform_arbitray_type_directives(&mut ctx, mem::take(&mut type_def.directives));
        }
    }

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

fn emit_interface_after_directives(mut ctx: Context<'_>) {
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

                let type_def = &mut ctx.out.type_definitions[usize::from(object.type_definition_id)];

                for subgraph_id in ctx
                    .subgraphs
                    .subgraphs_implementing_interface(implementee_name, implementer_name)
                {
                    type_def.directives.push(federated::Directive::JoinImplements(
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

                let type_def = &mut ctx.out.type_definitions[usize::from(interface.type_definition_id)];

                for subgraph_id in ctx
                    .subgraphs
                    .subgraphs_implementing_interface(implementee_name, implementer_name)
                {
                    type_def.directives.push(federated::Directive::JoinImplements(
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

        // Sort the fields by name.
        fields.sort_by(|a, b| ctx[a.field_name].cmp(&ctx[b.field_name]));

        for FieldIr {
            field_name,
            field_type,
            arguments,
            description,
            ..
        } in fields.drain(..)
        {
            let r#type = ctx.insert_field_type(ctx.subgraphs.walk(field_type));
            let field = federated::Field {
                name: field_name,
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
                }) => {
                    let selection_field = ctx.insert_string(ctx.subgraphs.walk(*field));
                    let field_id = ctx.selection_map[&(target, selection_field)];
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
                subgraphs::Selection::InlineFragment { on, subselection } => {
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

fn emit_subgraphs(ctx: &mut Context<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraphs() {
        let name = ctx.insert_string(subgraph.name());
        let url = ctx.insert_string(subgraph.url());
        ctx.out.subgraphs.push(federated::Subgraph { name, url });
    }
}
