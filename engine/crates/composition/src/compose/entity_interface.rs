use super::*;
use crate::composition_ir as ir;

pub(crate) fn merge_entity_interface_definitions<'a>(
    ctx: &mut Context<'a>,
    first: DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    let interface_name = first.name();
    let composed_directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    let interface_defs = || definitions.iter().filter(|def| def.kind() == DefinitionKind::Interface);
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
    let interface_id = ctx.insert_interface(interface_name, description, composed_directives);

    let mut fields = BTreeMap::new();

    for field in interface_def.fields() {
        fields.entry(field.name().id).or_insert_with(|| {
            let arguments = translate_arguments(field, ctx);

            ir::FieldIr {
                parent_definition: federated::Definition::Interface(interface_id),
                field_name: field.name().id,
                field_type: field.r#type().id,
                arguments,
                resolvable_in: Vec::new(),
                provides: Vec::new(),
                requires: Vec::new(),
                composed_directives: federated::NO_DIRECTIVES,
                overrides: Vec::new(),
                description: field.description().map(|description| ctx.insert_string(description.id)),
            }
        });
    }

    // All objects implementing that interface in the subgraph must have the same key.
    let Some(expected_key) = interface_def.entity_keys().next() else {
        ctx.diagnostics.push_fatal(format!(
            "The entity interface `{}` is missing a key in the `{}` subgraph.",
            first.name().as_str(),
            interface_def.subgraph().name().as_str(),
        ));
        return;
    };

    ctx.insert_interface_resolvable_key(interface_id, expected_key, false);

    // Each object has to have @interfaceObject and the same key as the entity interface.
    for definition in definitions.iter().filter(|def| def.kind() == DefinitionKind::Object) {
        if !definition.directives().interface_object() {
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
                first.name().as_str(),
                definition.subgraph().name().as_str(),
            ));
        }

        for entity_key in definition.entity_keys().filter(|key| key.is_resolvable()) {
            ctx.insert_interface_resolvable_key(interface_id, entity_key, true);
        }

        for field in definition.fields() {
            let description = field.description().map(|description| ctx.insert_string(description.id));
            fields.entry(field.name().id).or_insert_with(|| ir::FieldIr {
                parent_definition: federated::Definition::Interface(interface_id),
                field_name: field.name().id,
                field_type: field.r#type().id,
                arguments: translate_arguments(field, ctx),
                resolvable_in: vec![graphql_federated_graph::SubgraphId(definition.subgraph_id().idx())],
                provides: Vec::new(),
                requires: Vec::new(),
                composed_directives: federated::NO_DIRECTIVES,
                overrides: Vec::new(),
                description,
            });
        }
    }

    let field_ids: Vec<_> = fields.into_values().map(|field| ctx.insert_field(field)).collect();

    // Contribute the interface fields from the interface object definitions to the implementer of
    // that interface.
    for object in interface_def.subgraph().interface_implementers(first.name().id) {
        match object.entity_keys().next() {
            Some(key) if key.fields() == expected_key.fields() => (),
            Some(_) => ctx.diagnostics.push_fatal(format!(
                "[{}] The object type `{}` is annotated with @interfaceObject but has a different key than the entity interface `{}`.",
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
        for field_id in &field_ids {
            ctx.insert_object_field_from_entity_interface(object_name, *field_id);
        }
    }
}

fn translate_arguments(
    field: subgraphs::Walker<'_, (subgraphs::FieldId, subgraphs::FieldTuple)>,
    ctx: &mut ComposeContext<'_>,
) -> federated::InputValueDefinitions {
    let mut ids: Option<federated::InputValueDefinitions> = None;
    for arg in field.arguments() {
        let directives = collect_composed_directives(std::iter::once(arg.directives()), ctx);
        let name = ctx.insert_string(arg.name().id);
        let id = ctx.insert_input_value_definition(ir::InputValueDefinitionIr {
            name,
            r#type: arg.r#type().id,
            directives,
            description: None,
        });

        if let Some((_start, len)) = &mut ids {
            *len += 1;
        } else {
            ids = Some((id, 1));
        }
    }

    ids.unwrap_or(federated::NO_INPUT_VALUE_DEFINITION)
}
