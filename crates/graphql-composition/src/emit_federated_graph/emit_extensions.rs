use super::*;

pub(super) fn emit_extensions(ctx: &mut Context<'_>, ir: &CompositionIr) {
    let extensions_from_subgraphs = ctx.subgraphs.iter_extensions();

    if extensions_from_subgraphs.len() == 0 {
        return;
    }

    let namespace = ctx.insert_str("extension");
    let name = ctx.insert_str("Link");

    let extension_link_enum_id = ctx.out.push_enum_definition(federated::EnumDefinitionRecord {
        namespace: Some(namespace),
        name,
        directives: vec![],
        description: None,
    });

    for extension in extensions_from_subgraphs {
        let url = ctx.insert_string(ctx.subgraphs.walk(extension.url));

        let extension_name_str = ctx.subgraphs.walk(extension.name).as_str();
        let mut value = String::with_capacity(extension_name_str.len());

        for char in extension_name_str.chars() {
            match char {
                '-' => value.push('_'),
                _ => value.push(char.to_ascii_uppercase()),
            }
        }

        let value = ctx.insert_str(&value);

        let schema_directives: Vec<federated::ExtensionLinkSchemaDirective> =
            iter_directives_from_extension(extension.id, ctx.subgraphs, &ir.linked_schema_to_extension)
                .map(|(subgraph_id, directive)| {
                    let arguments = directive
                        .arguments
                        .iter()
                        .map(|(name, value)| (ctx.insert_string(ctx.subgraphs.walk(*name)), ctx.insert_value(value)))
                        .collect();

                    federated::ExtensionLinkSchemaDirective {
                        subgraph_id: subgraph_id.idx().into(),
                        name: ctx.insert_string(ctx.subgraphs.walk(directive.name)),
                        arguments: Some(arguments),
                    }
                })
                .collect();

        let enum_value_id = ctx.out.push_enum_value(federated::EnumValueRecord {
            enum_id: extension_link_enum_id,
            value,
            directives: vec![],
            description: None,
        });

        ctx.out.push_extension(federated::Extension {
            enum_value: enum_value_id,
            url,
            schema_directives,
        });
    }
}

fn iter_directives_from_extension<'a>(
    extension_id: subgraphs::ExtensionId,
    subgraphs: &'a Subgraphs,
    linked_schema_to_extension: &'a [(subgraphs::LinkedSchemaId, subgraphs::ExtensionId)],
) -> impl Iterator<Item = &'a (subgraphs::SubgraphId, subgraphs::ExtraDirectiveRecord)> {
    subgraphs
        .iter_extra_directives_on_schema_definition_or_extensions()
        .filter(move |(_, directive)| {
            let subgraphs::DirectiveProvenance::Linked {
                linked_schema_id,
                is_composed_directive: _,
            } = directive.provenance
            else {
                return false;
            };

            let Ok(idx) = linked_schema_to_extension.binary_search_by_key(&linked_schema_id, |(id, _)| *id) else {
                return false;
            };

            let (_, found_extension_id) = linked_schema_to_extension[idx];

            extension_id == found_extension_id
        })
}
