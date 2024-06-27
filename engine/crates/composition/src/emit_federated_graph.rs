mod attach_argument_selection;
mod context;
mod emit_fields;
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
    let __schema = ir.strings.insert("__schema");
    let __type = ir.strings.insert("__type");

    let mut out = federated::FederatedGraphV3 {
        enums: mem::take(&mut ir.enums),
        enum_values: mem::take(&mut ir.enum_values),
        objects: mem::take(&mut ir.objects),
        interfaces: mem::take(&mut ir.interfaces),
        unions: mem::take(&mut ir.unions),
        scalars: mem::take(&mut ir.scalars),
        input_objects: mem::take(&mut ir.input_objects),
        directives: mem::take(&mut ir.directives),
        root_operation_types: RootOperationTypes {
            query: ir.query_type.unwrap(),
            mutation: ir.mutation_type,
            subscription: ir.subscription_type,
        },

        subgraphs: vec![],
        fields: vec![],
        input_value_definitions: vec![],
        strings: vec![],
        authorized_directives: vec![],
        field_authorized_directives: vec![],
        object_authorized_directives: vec![],
    };

    let mut ctx = Context::new(&mut ir, subgraphs, &mut out);

    emit_subgraphs(&mut ctx);
    emit_interface_impls(&mut ctx);
    emit_input_value_definitions(&ir.input_value_definitions, &mut ctx);
    emit_fields(
        mem::take(&mut ir.fields),
        &ir.object_fields_from_entity_interfaces,
        __schema,
        __type,
        &mut ctx,
    );
    emit_union_members(&ir.union_members, &mut ctx);
    emit_keys(&ir.keys, &mut ctx);
    emit_authorized_directives(&ir, &mut ctx);

    drop(ctx);

    federated::FederatedGraph::V3(out)
}

fn emit_authorized_directives(ir: &CompositionIr, ctx: &mut Context<'_>) {
    for (object_id, authorized) in &ir.object_authorized_directives {
        let fields = todo!();

        let arguments = todo!();

        let rule = ctx.insert_string(ctx.subgraphs.walk(authorized.rule));
        let metadata = authorized.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));

        let authorized_directive_id = ctx
            .out
            .authorized_directives
            .push_return_idx(federated::AuthorizedDirective {
                rule,
                fields,
                arguments,
                metadata,
            });

        ctx.out
            .object_authorized_directives
            .push((object_id, authorized_directive_id));
    }
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
                r#type: ctx.insert_field_type(ctx.subgraphs.walk(*r#type)),
                directives: *directives,
                description: *description,
            },
        )
        .collect()
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

fn emit_fields<'a>(
    ir_fields: Vec<FieldIr>,
    object_fields_from_entity_interfaces: &BTreeSet<(federated::StringId, federated::FieldId)>,
    __schema: federated::StringId,
    __type: federated::StringId,
    ctx: &mut Context<'a>,
) {
    // We have to accumulate the `@provides`, `@requires` and `@authorized` and delay emitting them because
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
    let mut field_authorized: Vec<(
        federated::FieldId,
        federated::Definition,
        &subgraphs::AuthorizedDirective,
    )> = Vec::new();

    emit_fields::for_each_field_group(&ir_fields, |definition, fields| {
        let mut start_field_id = None;
        let mut end_field_id = None;

        if let federated::Definition::Object(id) = definition {
            let object_name = ctx.out.objects[id.0].name;
            let fields_from_entity_interfaces = object_fields_from_entity_interfaces
                .range((object_name, federated::FieldId(0))..(object_name, federated::FieldId(usize::MAX)))
                .map(|(_, field_id)| ir_fields[field_id.0].clone());

            fields.extend(fields_from_entity_interfaces);
        }

        // Sort the fields by name.
        fields.sort_by(|a, b| {
            ctx.subgraphs
                .walk(a.field_name)
                .as_str()
                .cmp(ctx.subgraphs.walk(b.field_name).as_str())
        });

        for FieldIr {
            parent_definition: _,
            field_name,
            field_type,
            arguments,
            resolvable_in,
            provides,
            requires,
            composed_directives,
            overrides,
            description,
            authorized_directives,
        } in fields.drain(..)
        {
            let r#type = ctx.insert_field_type(ctx.subgraphs.walk(field_type));
            let field_name = ctx.insert_string(ctx.subgraphs.walk(field_name));

            let field = federated::Field {
                name: field_name,
                r#type,
                arguments,
                overrides,

                provides: Vec::new(),
                requires: Vec::new(),
                resolvable_in,
                composed_directives,
                description,
            };

            let field_id = federated::FieldId(ctx.out.fields.push_return_idx(field));

            start_field_id = start_field_id.or(Some(field_id));
            end_field_id = Some(field_id);

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
                field_provides.push((field_id, subgraph_id, definition, provides));
            }

            for (subgraph_id, requires) in requires.iter().filter_map(|field_id| {
                let field = ctx.subgraphs.walk_field(*field_id);
                field.directives().requires().map(|requires| {
                    (
                        federated::SubgraphId(field.parent_definition().subgraph_id().idx()),
                        requires,
                    )
                })
            }) {
                field_requires.push((field_id, subgraph_id, definition, requires));
            }

            for authorized in authorized_directives
                .iter()
                .filter_map(|field_id| ctx.subgraphs.walk_field(*field_id).directives().authorized())
            {
                field_authorized.push((field_id, definition, authorized));
            }

            let selection_map_key = (definition, field_name);
            ctx.selection_map.insert(selection_map_key, field_id);
        }

        let fields = start_field_id
            .zip(end_field_id)
            .map(|(start, end)| federated::Fields {
                start,
                end: federated::FieldId(end.0 + 1),
            })
            .unwrap_or(federated::NO_FIELDS);

        match definition {
            federated::Definition::Object(id) if id == ctx.out.root_operation_types.query => {
                // Here we want to reserve two spots for the __schema and __type fields used for introspection.

                let extra_fields = [__schema, __type].map(|name| federated::Field {
                    name,
                    // Dummy type
                    r#type: federated::Type {
                        wrapping: federated::Wrapping::new(false),
                        definition,
                    },
                    arguments: federated::NO_INPUT_VALUE_DEFINITION,
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: federated::NO_DIRECTIVES,
                    description: None,
                });

                ctx.out.fields.extend_from_slice(&extra_fields);
                ctx.out.objects[id.0].fields = federated::Fields {
                    start: fields.start,
                    end: federated::FieldId(fields.end.0 + 2),
                };
            }
            federated::Definition::Object(id) => {
                ctx.out.objects[id.0].fields = fields;
            }
            federated::Definition::Interface(id) => {
                ctx.out.interfaces[id.0].fields = fields;
            }
            _ => unreachable!(),
        }
    });

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

    for (field_id, definition, authorized) in field_authorized {
        let fields = authorized
            .fields
            .as_ref()
            .map(|fields| attach_selection(fields, definition, ctx));
        let metadata = authorized.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));

        let arguments = authorized
            .arguments
            .as_ref()
            .map(|args| attach_argument_selection::attach_argument_selection(args, field_id, ctx));

        let rule = ctx.insert_string(ctx.subgraphs.walk(authorized.rule));

        let idx = ctx
            .out
            .authorized_directives
            .push_return_idx(federated::AuthorizedDirective {
                rule,
                fields,
                arguments,
                metadata,
            });

        ctx.out
            .field_authorized_directives
            .push((field_id, federated::AuthorizedDirectiveId(idx)));
    }
}

fn emit_union_members(ir_members: &BTreeSet<(federated::StringId, federated::StringId)>, ctx: &mut Context<'_>) {
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
            let field_ty = ctx.out[field].r#type.definition;
            let field_arguments = ctx.out[field].arguments;
            let (federated::InputValueDefinitionId(field_arguments_start), _) = field_arguments;
            let arguments = selection
                .arguments
                .iter()
                .map(|(name, value)| {
                    // Here we assume the arguments are validated previously.
                    let arg_name = ctx.insert_string(ctx.subgraphs.walk(*name));
                    let argument = ctx.out[field_arguments]
                        .iter()
                        .position(|arg| arg.name == arg_name)
                        .map(|idx| federated::InputValueDefinitionId(field_arguments_start + idx))
                        .unwrap();
                    let value = ctx.insert_value(value);
                    (argument, value)
                })
                .collect();

            federated::FieldSetItem {
                field,
                arguments,
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
