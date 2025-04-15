use itertools::Itertools;

use crate::{
    ExtensionDirectiveId, ExtensionDirectiveRecord, SubgraphId,
    builder::{Error, GraphBuilder, sdl},
};

use super::{LoadedExtension, LoadedExtensionOrCompositeSchema};

pub(crate) fn ingest_extension_schema_directives(builder: &mut GraphBuilder<'_>) -> Result<(), Error> {
    for (name, ext) in builder.sdl.extensions.iter() {
        let LoadedExtensionOrCompositeSchema::Extension(extension) = builder.extensions.get(*name) else {
            continue;
        };
        for (directive, span) in &ext.directives {
            let subgraph_id = builder.subgraphs.try_get(directive.graph, *span)?;
            let id = builder
                .ingest_extension_directive(
                    sdl::SdlDefinition::SchemaDirective(subgraph_id),
                    subgraph_id,
                    extension,
                    directive.name,
                    directive.arguments,
                )
                .map_err(|txt| (txt, *span))?;
            match subgraph_id {
                SubgraphId::GraphqlEndpoint(subgraph_id) => {
                    builder.subgraphs[subgraph_id].schema_directive_ids.push(id);
                }
                SubgraphId::Virtual(subgraph_id) => {
                    builder.subgraphs[subgraph_id].schema_directive_ids.push(id);
                }
                SubgraphId::Introspection => unreachable!(),
            }
        }
    }
    Ok(())
}

impl<'a> GraphBuilder<'a> {
    pub(crate) fn ingest_extension_directive(
        &mut self,
        current_definition: sdl::SdlDefinition<'a>,
        subgraph_id: SubgraphId,
        extension: &'a LoadedExtension<'a>,
        name: &str,
        arguments: Option<sdl::ConstValue<'a>>,
    ) -> Result<ExtensionDirectiveId, String> {
        let directive_name_id = self.ingest_str(name);

        let Some(sdl) = &extension.sdl else {
            return Err(format!(
                "At site {}, extension '{}' does not define any GraphQL definitions, but a directive @{name} was found",
                current_definition.to_site_string(self),
                extension.manifest.id
            ));
        };

        let Some(definition) = sdl.doc.definitions().find_map(|def| match def {
            cynic_parser::type_system::Definition::Directive(dir) if dir.name() == name => Some(dir),
            _ => None,
        }) else {
            return Err(format!(
                "At site {}, unknown extension directive @{name} for extension '{}'",
                current_definition.to_site_string(self),
                extension.manifest.id
            ));
        };

        let directive_type = extension.manifest.get_directive_type(name);

        let location = current_definition.location();
        if definition.locations().all(|loc| loc != location) {
            return Err(format!(
                "At site {}, extension {} directive @{name} used in the wrong location {}, expected one of: {}",
                current_definition.to_site_string(self),
                extension.manifest.id,
                location.as_str(),
                definition.locations().map(|loc| loc.as_str()).join(", ")
            ));
        }

        if directive_type.is_selection_set_resolver() {
            let id = match subgraph_id {
                SubgraphId::Virtual(id) => id,
                SubgraphId::Introspection => unreachable!(),
                SubgraphId::GraphqlEndpoint(id) => {
                    return Err(format!(
                        "At site {}, resolver extension {}' directive @{name} can only be used on virtual graphs, '{}' isn't one.",
                        current_definition.to_site_string(self),
                        extension.manifest.id,
                        &self.ctx[self.ctx[id].subgraph_name_id]
                    ));
                }
            };

            if let Some(other_id) =
                self.virtual_subgraph_to_selection_set_resolver[usize::from(id)].filter(|id| *id != extension.id)
            {
                return Err(format!(
                    "At site {}, Selection Set Resolver extension {} cannot be mixed with other resolvers in subgraph '{}', found {}",
                    current_definition.to_site_string(self),
                    extension.manifest.id,
                    self.ctx[self.ctx[id].subgraph_name_id].clone(),
                    self.ctx[other_id].manifest.id.clone(),
                ));
            }
            self.virtual_subgraph_to_selection_set_resolver[usize::from(id)] = Some(extension.id);
        }

        let (argument_ids, requirements_record) = self
            .coerce_extension_directive_arguments(current_definition, sdl, definition, arguments)
            .map_err(|err| {
                format!(
                    "At site {}, for the extension '{}' directive @{name}: {err}",
                    current_definition.to_site_string(self),
                    extension.manifest.id,
                )
            })?;

        let record = ExtensionDirectiveRecord {
            subgraph_id,
            extension_id: extension.id,
            name_id: directive_name_id,
            ty: directive_type,
            argument_ids,
            requirements_record,
        };
        self.graph.extension_directives.push(record);
        let id = (self.graph.extension_directives.len() - 1).into();
        Ok(id)
    }
}
