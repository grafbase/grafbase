mod attach_argument_selection;
mod context;
mod emit_fields;
mod field_types_map;

use self::context::Context;
use crate::{
    composition_ir::{self as ir, CompositionIr, FieldIr, InputValueDefinitionIr, KeyIr},
    subgraphs::{self, SubgraphId},
    Subgraphs, VecExt,
};
use graphql_federated_graph as federated;
use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(mut ir: CompositionIr, subgraphs: &Subgraphs) -> federated::FederatedGraph {
    let mut out = federated::FederatedGraph {
        type_definitions: mem::take(&mut ir.type_definitions),
        enum_values: mem::take(&mut ir.enum_values),
        objects: mem::take(&mut ir.objects),
        interfaces: mem::take(&mut ir.interfaces),
        unions: mem::take(&mut ir.unions),
        input_objects: mem::take(&mut ir.input_objects),
        root_operation_types: federated::RootOperationTypes {
            query: ir.query_type.unwrap(),
            mutation: ir.mutation_type,
            subscription: ir.subscription_type,
        },

        directives: vec![],
        subgraphs: vec![],
        fields: vec![],
        input_value_definitions: vec![],
        strings: vec![],
        authorized_directives: vec![],
        field_authorized_directives: vec![],
        object_authorized_directives: vec![],
        interface_authorized_directives: vec![],
    };

    let mut ctx = Context::new(&mut ir, subgraphs, &mut out);

    emit_directives(&mut ir.directives, &mut ctx);
    emit_subgraphs(&mut ctx);
    emit_interface_impls(&mut ctx);
    emit_input_value_definitions(&ir.input_value_definitions, &mut ctx);
    emit_fields(
        mem::take(&mut ir.fields),
        &ir.object_fields_from_entity_interfaces,
        &mut ctx,
    );
    emit_union_members(&ir.union_members, &ir.union_join_members, &mut ctx);
    emit_keys(&ir.keys, &mut ctx);
    emit_authorized_directives(&ir, &mut ctx);

    drop(ctx);

    out
}

fn emit_directives(ir: &mut Vec<ir::Directive>, ctx: &mut Context<'_>) {
    ctx.out.directives.reserve(ir.len());

    for directive in ir.drain(..) {
        let converted = match directive {
            ir::Directive::Authenticated => federated::Directive::Authenticated,
            ir::Directive::Deprecated { reason } => federated::Directive::Deprecated { reason },
            ir::Directive::Inaccessible => federated::Directive::Inaccessible,
            ir::Directive::Policy(policies) => federated::Directive::Policy(policies),
            ir::Directive::RequiresScopes(scopes) => federated::Directive::RequiresScopes(scopes),
            ir::Directive::Cost { weight } => federated::Directive::Cost { weight },
            ir::Directive::Other { name, arguments } => federated::Directive::Other {
                name,
                arguments: arguments
                    .into_iter()
                    .map(|(name, value)| (name, ctx.insert_value(&value)))
                    .collect(),
            },
        };

        ctx.out.directives.push(converted);
    }
}

fn emit_authorized_directives(ir: &CompositionIr, ctx: &mut Context<'_>) {
    for (object_id, authorized) in &ir.object_authorized_directives {
        let authorized = ctx.subgraphs.walk(*authorized).authorized().unwrap();
        let metadata = authorized.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));
        let fields = authorized
            .fields
            .as_ref()
            .map(|fields| attach_selection(fields, federated::Definition::Object(*object_id), ctx));

        let authorized_directive_id = ctx
            .out
            .authorized_directives
            .push_return_idx(federated::AuthorizedDirective {
                fields,
                node: None,
                arguments: None,
                metadata,
            });

        let authorized_directive_id = federated::AuthorizedDirectiveId::from(authorized_directive_id);

        ctx.out
            .object_authorized_directives
            .push((*object_id, authorized_directive_id));
    }

    for (interface_id, authorized) in &ir.interface_authorized_directives {
        let authorized = ctx.subgraphs.walk(*authorized).authorized().unwrap();
        let metadata = authorized.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));
        let fields = authorized
            .fields
            .as_ref()
            .map(|fields| attach_selection(fields, federated::Definition::Interface(*interface_id), ctx));

        let authorized_directive_id = ctx
            .out
            .authorized_directives
            .push_return_idx(federated::AuthorizedDirective {
                fields,
                node: None,
                arguments: None,
                metadata,
            });

        let authorized_directive_id = federated::AuthorizedDirectiveId::from(authorized_directive_id);

        ctx.out
            .interface_authorized_directives
            .push((*interface_id, authorized_directive_id));
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
                 default,
             }| {
                let r#type = ctx.insert_field_type(ctx.subgraphs.walk(*r#type));
                let default = default
                    .as_ref()
                    .map(|default| ctx.insert_value_with_type(default, r#type.definition.as_enum()));

                federated::InputValueDefinition {
                    name: *name,
                    r#type,
                    directives: *directives,
                    description: *description,
                    default,
                }
            },
        )
        .collect()
}

fn emit_interface_impls(ctx: &mut Context<'_>) {
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
                    object.join_implements.push((
                        graphql_federated_graph::SubgraphId::from(subgraph_id.idx()),
                        implementee,
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
                    interface.join_implements.push((
                        graphql_federated_graph::SubgraphId::from(subgraph_id.idx()),
                        implementee,
                    ));
                }
            }
            _ => unreachable!(),
        }
    }
}

fn emit_fields<'a>(
    ir_fields: Vec<FieldIr>,
    object_fields_from_entity_interfaces: &BTreeSet<(federated::StringId, federated::FieldId)>,
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

    struct AuthorizedField<'a> {
        parent: federated::Definition,
        field_id: federated::FieldId,
        output: federated::Definition,
        directive: &'a subgraphs::AuthorizedDirective,
    }
    let mut field_authorized: Vec<AuthorizedField<'_>> = Vec::new();

    emit_fields::for_each_field_group(&ir_fields, |definition, fields| {
        let mut start_field_id = None;
        let mut end_field_id = None;

        if let federated::Definition::Object(id) = definition {
            let object_name = ctx.out.at(id).then(|obj| obj.type_definition_id).name;
            let fields_from_entity_interfaces = object_fields_from_entity_interfaces
                .range((object_name, federated::FieldId::from(0))..(object_name, federated::FieldId::from(usize::MAX)))
                .map(|(_, field_id)| ir_fields[usize::from(*field_id)].clone());

            fields.extend(fields_from_entity_interfaces);
        }

        // Sort the fields by name.
        fields.sort_by(|a, b| ctx[a.field_name].cmp(&ctx[b.field_name]));

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
            let output_definition = r#type.definition;

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

            let field_id = federated::FieldId::from(ctx.out.fields.push_return_idx(field));

            start_field_id = start_field_id.or(Some(field_id));
            end_field_id = Some(field_id);

            for (subgraph_id, definition, provides) in provides.iter().filter_map(|field_id| {
                let field = ctx.subgraphs.walk_field(*field_id);
                field.directives().provides().map(|provides| {
                    let field_type_name = ctx.insert_string(field.r#type().type_name());
                    (
                        federated::SubgraphId::from(field.parent_definition().subgraph_id().idx()),
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
                        federated::SubgraphId::from(field.parent_definition().subgraph_id().idx()),
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
                field_authorized.push(AuthorizedField {
                    parent: definition,
                    field_id,
                    output: output_definition,
                    directive: authorized,
                });
            }

            let selection_map_key = (definition, field_name);
            ctx.selection_map.insert(selection_map_key, field_id);
        }

        let fields = start_field_id
            .zip(end_field_id)
            .map(|(start, end)| federated::Fields {
                start,
                end: federated::FieldId::from(usize::from(end) + 1),
            })
            .unwrap_or(federated::NO_FIELDS);

        match definition {
            federated::Definition::Object(id) => {
                ctx.out.objects[usize::from(id)].fields = fields;
            }
            federated::Definition::Interface(id) => {
                ctx.out.interfaces[usize::from(id)].fields = fields;
            }
            _ => unreachable!(),
        }
    });

    for (field_id, subgraph_id, definition, provides) in field_provides {
        let fields = attach_selection(provides, definition, ctx);
        ctx.out.fields[usize::from(field_id)]
            .provides
            .push(federated::FieldProvides { subgraph_id, fields });
    }

    for (field_id, subgraph_id, definition, requires) in field_requires {
        let fields = attach_selection(requires, definition, ctx);
        ctx.out.fields[usize::from(field_id)]
            .requires
            .push(federated::FieldRequires { subgraph_id, fields });
    }

    for AuthorizedField {
        parent,
        field_id,
        output,
        directive,
    } in field_authorized
    {
        let fields = directive
            .fields
            .as_ref()
            .map(|field_set| attach_selection(field_set, parent, ctx));
        let node = directive
            .node
            .as_ref()
            .map(|field_set| attach_selection(field_set, output, ctx));
        let metadata = directive.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));

        let arguments = directive
            .arguments
            .as_ref()
            .map(|args| attach_argument_selection::attach_argument_selection(args, field_id, ctx));

        let idx = ctx
            .out
            .authorized_directives
            .push_return_idx(federated::AuthorizedDirective {
                fields,
                node,
                arguments,
                metadata,
            });

        ctx.out
            .field_authorized_directives
            .push((field_id, federated::AuthorizedDirectiveId::from(idx)));
    }
}

fn emit_union_members(
    ir_members: &BTreeSet<(federated::StringId, federated::StringId)>,
    ir_join_members: &BTreeMap<(federated::StringId, federated::StringId), Vec<SubgraphId>>,
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

            for subgraph_id in ir_join_members.get(&(*union_name, *member)).into_iter().flatten() {
                let subgraph_id = federated::SubgraphId::from(subgraph_id.idx());
                union.join_members.insert((subgraph_id, object_id));
            }
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
            subgraph_id: federated::SubgraphId::from(key.parent_definition().subgraph_id().idx()),
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
                    let field = ctx.selection_map[&(target, selection_field)];
                    let field_ty = ctx.out[field].r#type.definition;
                    let field_arguments = ctx.out[field].arguments;
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

                    federated::Selection::Field {
                        field,
                        arguments,
                        subselection: attach_selection(subselection, field_ty, ctx),
                    }
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
