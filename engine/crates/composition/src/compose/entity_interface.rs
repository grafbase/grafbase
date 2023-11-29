use super::*;
use crate::composition_ir as ir;

pub(crate) fn merge_entity_interface_definitions(
    ctx: &mut Context<'_>,
    first: DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let interface_name = first.name();
    let is_inaccessible = definitions.iter().any(|definition| definition.is_inaccessible());

    let interface_defs = || definitions.iter().filter(|def| def.kind() == DefinitionKind::Interface);
    let mut interfaces = interface_defs();

    let Some(interface_def) = interfaces.next() else {
        ctx.diagnostics.push_fatal(format!(
            "The entity interface `{}` is not defined as an interface in any subgraph.",
            interface_name.as_str()
        ));
        return;
    };

    // More than one interface in subgraphs.
    if interfaces.next().is_some() {
        let all_implementers: BTreeSet<_> = interface_defs()
            .flat_map(|interface| {
                interface
                    .subgraph()
                    .interface_implementers(interface_name.id)
                    .map(|def| def.name().id)
            })
            .collect();

        // All subsequent interfaces must have the same implementers.
        for interface in interface_defs() {
            let implementers: BTreeSet<_> = interface
                .subgraph()
                .interface_implementers(interface_name.id)
                .map(|def| def.name().id)
                .collect();

            if implementers != all_implementers {
                let subgraph_name = interface.subgraph().name().as_str();
                let interface_name = interface_name.as_str();
                let implementer_names = all_implementers
                    .difference(&implementers)
                    .map(|id| ctx.subgraphs.walk(*id).as_str())
                    .join(", ");
                ctx.diagnostics.push_fatal(format!(
                r#"[{subgraph_name}]: Interface type "{interface_name}" has a resolvable key in subgraph "{subgraph_name}" but that subgraph is missing some of the supergraph implementation types of "{interface_name}". Subgraph "{subgraph_name}" should define types {implementer_names}."#
                ));
            }

            if interface.is_interface_object() {
                ctx.diagnostics.push_fatal(format!(
                    "[{}] The @interfaceObject directive is not valid on interfaces (on `{}`).",
                    interface.subgraph().name().as_str(),
                    interface_name.as_str(),
                ));
            }
        }
    }

    let interface_id = ctx.insert_interface(interface_name, is_inaccessible);

    let mut fields = BTreeMap::new();

    for field in interface_def.fields() {
        fields.entry(field.name().id).or_insert_with(|| ir::FieldIr {
            parent_name: interface_def.name().id,
            field_name: field.name().id,
            field_type: field.r#type().id,
            arguments: field
                .arguments()
                .map(|arg| ir::ArgumentIr {
                    argument_name: arg.argument_name().id,
                    argument_type: arg.argument_type().id,
                    composed_directives: if arg.is_inaccessible() {
                        vec![federated::Directive {
                            name: ctx.insert_static_str("inaccessible"),
                            arguments: Vec::new(),
                        }]
                    } else {
                        Vec::new()
                    },
                })
                .collect(),
            resolvable_in: None,
            provides: Vec::new(),
            requires: Vec::new(),
            composed_directives: Vec::new(),
            overrides: Vec::new(),
        });
    }

    // All objects implementing that interface in the subgraph must have the same key.
    let Some(expected_key) = interface_def.entity_keys().next() else {
        ctx.diagnostics.push_fatal(format!(
            "The entity interface `{}` is missing a key in the `{}` subgraph.",
            interface_name.as_str(),
            interface_def.subgraph().name().as_str(),
        ));
        return;
    };

    ctx.insert_interface_resolvable_key(interface_id, expected_key.id, false);

    // Each object has to have @interfaceObject and the same key as the entity interface.
    for definition in definitions.iter().filter(|def| def.kind() == DefinitionKind::Object) {
        if !definition.is_interface_object() {
            ctx.diagnostics.push_fatal(format!(
                "`{}` is an entity interface but the object type `{}` is missing the @interfaceObject directive in the `{}` subgraph.",
                definition.name().as_str(),
                definition.name().as_str(),
                definition.subgraph().name().as_str(),
            ));
        }

        if definition.entity_keys().next().is_none() {
            ctx.diagnostics.push_fatal(format!(
                "The object type `{}` is annotated with @interfaceObject but missing a key in the `{}` subgraph.",
                interface_name.as_str(),
                definition.subgraph().name().as_str(),
            ));
        }

        for entity_key in definition.entity_keys().filter(|key| key.is_resolvable()) {
            ctx.insert_interface_resolvable_key(interface_id, entity_key.id, true);
        }

        for field in definition.fields() {
            fields.entry(field.name().id).or_insert_with(|| ir::FieldIr {
                parent_name: definition.name().id,
                field_name: field.name().id,
                field_type: field.r#type().id,
                arguments: field
                    .arguments()
                    .map(|arg| ir::ArgumentIr {
                        argument_name: arg.argument_name().id,
                        argument_type: arg.argument_type().id,
                        composed_directives: if arg.is_inaccessible() {
                            vec![federated::Directive {
                                name: ctx.insert_static_str("inaccessible"),
                                arguments: Vec::new(),
                            }]
                        } else {
                            Vec::new()
                        },
                    })
                    .collect(),
                resolvable_in: Some(graphql_federated_graph::SubgraphId(definition.subgraph().id.idx())),
                provides: Vec::new(),
                requires: Vec::new(),
                composed_directives: Vec::new(),
                overrides: Vec::new(),
            });
        }
    }

    // Contribute the interface fields from the interface object definitions to the implementer of
    // that interface.
    for object in interface_def.subgraph().interface_implementers(interface_name.id) {
        match object.entity_keys().next() {
            Some(key) if key.fields() == expected_key.fields() => (),
            Some(_) => ctx.diagnostics.push_fatal(format!(
                "[{}] The object type `{}` is annotated with @interfaceObject but has a different key than the entity interface `{}`.",
                object.subgraph().name().as_str(),
                object.name().as_str(),
                interface_name.as_str(),
            )),
            None => ctx.diagnostics.push_fatal(format!(
                "[{}] The object type `{}` is annotated with @interfaceObject but missing a key.",
                object.subgraph().name().as_str(),
                object.name().as_str(),
            )),
        }

        for ir::FieldIr {
            parent_name: _,
            field_name,
            field_type,
            arguments,
            resolvable_in,
            provides,
            requires,
            composed_directives,
            overrides,
        } in fields.values()
        {
            ctx.insert_field(ir::FieldIr {
                parent_name: object.name().id,
                field_name: *field_name,
                field_type: *field_type,
                arguments: arguments.clone(),
                resolvable_in: *resolvable_in,
                provides: provides.clone(),
                requires: requires.clone(),
                composed_directives: composed_directives.clone(),
                overrides: overrides.clone(),
            });
        }
    }

    for field in fields.into_values() {
        ctx.insert_field(field);
    }
}
