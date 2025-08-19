use super::*;

pub(super) fn is_entity_interface(
    subgraphs: &subgraphs::Subgraphs,
    mut definitions: impl Iterator<Item = subgraphs::DefinitionId>,
) -> bool {
    // Take the first federation v2 definition.
    let Some(definition_id) = definitions.find(|def| {
        let subgraph_id = subgraphs.at(*def).subgraph_id;
        subgraphs.at(subgraph_id).federation_spec.is_apollo_v2()
    }) else {
        return false;
    };

    // Is it an entity interface object, or an interface with @key?
    let definition = &subgraphs.at(definition_id);

    match definition.kind {
        DefinitionKind::Object => definition.directives.interface_object(subgraphs),
        DefinitionKind::Interface => definition_id.keys(subgraphs).next().is_some(),
        _ => false,
    }
}

pub(crate) fn merge_entity_interface_definitions<'a>(
    ctx: &mut Context<'a>,
    first: DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    let interface_name = first.name();

    let interface_defs = || {
        definitions.iter().filter(|def| {
            def.kind() == DefinitionKind::Interface
                && ctx
                    .subgraphs
                    .at(ctx.subgraphs[def.id].subgraph_id)
                    .federation_spec
                    .is_apollo_v2()
        })
    };
    let mut interfaces = interface_defs();

    let Some(interface_def) = interfaces.next() else {
        ctx.diagnostics.push_fatal(format!(
            "The entity interface `{}` is not defined as an interface in any subgraph.",
            first.name().as_str()
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

            if interface.directives().interface_object() {
                ctx.diagnostics.push_fatal(format!(
                    "[{}] The @interfaceObject directive is not valid on interfaces (on `{}`).",
                    interface.subgraph().name().as_str(),
                    interface_name.as_str(),
                ));
            }
        }
    }

    let description = interface_def.description().map(|d| d.as_str());
    let interface_name = ctx.insert_string(interface_name.id);
    let directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);
    let interface_id = ctx.insert_interface(interface_name, description, directives);

    let Some(expected_key) = interface_def.entity_keys().next() else {
        ctx.diagnostics.push_fatal(format!(
            "The entity interface `{}` is missing a key in the `{}` subgraph.",
            first.name().as_str(),
            interface_def.subgraph().name().as_str(),
        ));
        return;
    };

    ctx.insert_interface_resolvable_key(interface_id, expected_key, false);

    // Each object in other subgraphs has to have @interfaceObject and the same key as the entity interface.
    for definition in definitions.iter().filter(|def| def.kind() == DefinitionKind::Object) {
        if !definition.directives().interface_object() {
            ctx.diagnostics.push_fatal(format!(
                "`{}` is an entity interface but the object type `{}` is missing the @interfaceObject directive in the `{}` subgraph.",
                definition.name().as_str(),
                definition.name().as_str(),
                definition.subgraph().name().as_str(),
            ));
        }

        match definition.entity_keys().next() {
            None => {
                ctx.diagnostics.push_fatal(format!(
                    "The object type `{}` is annotated with @interfaceObject but missing a key in the `{}` subgraph.",
                    first.name().as_str(),
                    definition.subgraph().name().as_str(),
                ));
            }
            Some(key) if key.fields() == expected_key.fields() => (),
            Some(_) => {
                ctx.diagnostics.push_fatal(format!(
                    "[{}] The object type `{}` is annotated with @interfaceObject but has a different key than the entity interface `{}`.",
                    definition.subgraph().name().as_str(),
                    definition.name().as_str(),
                    interface_def.name().as_str(),
                ));
            }
        }

        for entity_key in definition.entity_keys().filter(|key| key.is_resolvable()) {
            ctx.insert_interface_resolvable_key(interface_id, entity_key, true);
        }
    }

    let fields = object::compose_fields(ctx, definitions, interface_name);

    let fields_to_add: Vec<(subgraphs::StringId, _)> = fields
        .into_iter()
        .map(|mut field| {
            // Adding interface field.
            ctx.insert_field(field.clone());

            // Adding only the empty `@join__field` directive indicating it's coming from somewhere
            // else.
            field.directives = vec![ir::Directive::JoinEntityInterfaceField];
            (field.field_name, field)
        })
        .collect();

    // The fields of the entity interface are not only defined in the subgraph where the entity interface is an interface.
    // More fields are contributed by other subgraphs where there are objects with `@interfaceObject`. Those must be added now in all
    // the implementers of the interface as they won't have them in their definition.
    for object in interface_def.subgraph().interface_implementers(first.name().id) {
        match object.entity_keys().next() {
            Some(key) if key.fields() == expected_key.fields() => (),
            Some(_) => ctx.diagnostics.push_fatal(format!(
                "[{}] The object type `{}` implements the entity interface `{}` but does not have the same key. The key must match exactly.",
                object.subgraph().name().as_str(),
                object.name().as_str(),
                first.name().as_str(),
            )),
            None => ctx.diagnostics.push_fatal(format!(
                "[{}] The object type `{}` is annotated with @interfaceObject but missing a key.",
                object.subgraph().name().as_str(),
                object.name().as_str(),
            )),
        }

        let object_name = ctx.insert_string(object.name().id);

        let fields_to_add = fields_to_add
            .iter()
            // Avoid adding fields that are already present on the object by virtue of the object implementing the interface.
            .filter(|(name, _)| object.find_field(*name).is_none())
            .map(|(_, field_ir)| field_ir);

        for field_ir in fields_to_add {
            let mut field_ir = field_ir.clone();
            field_ir.parent_definition_name = object_name;
            ctx.insert_field(field_ir);
        }
    }
}
