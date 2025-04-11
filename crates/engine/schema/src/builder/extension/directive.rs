use crate::{
    BuildError, ExtensionDirectiveId, ExtensionDirectiveRecord, SubgraphId,
    builder::{
        GraphBuilder, SchemaLocation,
        error::{
            ExtensionDirectiveArgumentsError, ExtensionDirectiveLocationError,
            SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError,
        },
        sdl,
    },
};

use super::LoadedExtension;

pub(crate) fn ingest_extension_schema_directives(builder: &mut GraphBuilder<'_>) -> Result<(), BuildError> {
    for (name, ext) in builder.sdl.extensions.iter() {
        let extension = builder.extensions.try_get(*name)?;
        for directive in &ext.directives {
            let subgraph_id = builder.subgraphs.try_get(directive.graph)?;
            let id = builder.ingest_extension_directive(
                SchemaLocation::SchemaDirective(subgraph_id),
                subgraph_id,
                extension,
                directive.name,
                directive.arguments,
            )?;
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
        location: SchemaLocation<'a>,
        subgraph_id: SubgraphId,
        extension: &'a LoadedExtension<'a>,
        name: &str,
        arguments: Option<sdl::ConstValue<'a>>,
    ) -> Result<ExtensionDirectiveId, BuildError> {
        let directive_name_id = self.ingest_str(name);

        let Some(sdl) = &extension.sdl else {
            return Err(BuildError::MissingGraphQLDefinitions {
                id: extension.manifest.id.clone(),
                directive: name.to_string(),
            });
        };

        let Some(definition) = sdl.doc.definitions().find_map(|def| match def {
            cynic_parser::type_system::Definition::Directive(dir) if dir.name() == name => Some(dir),
            _ => None,
        }) else {
            return Err(BuildError::UnknownExtensionDirective {
                id: extension.manifest.id.clone(),
                directive: name.to_string(),
            });
        };

        let directive_type = extension.manifest.get_directive_type(name);

        let cynic_location = location.as_cynic_location();
        if definition.locations().all(|loc| loc != cynic_location) {
            return Err(BuildError::ExtensionDirectiveLocationError(Box::new(
                ExtensionDirectiveLocationError {
                    id: extension.manifest.id.clone(),
                    directive: name.to_string(),
                    location: cynic_location.as_str(),
                    expected: definition.locations().map(|loc| loc.as_str()).collect(),
                },
            )));
        }

        if directive_type.is_selection_set_resolver() {
            let id = match subgraph_id {
                SubgraphId::Virtual(id) => id,
                SubgraphId::Introspection => unreachable!(),
                SubgraphId::GraphqlEndpoint(id) => {
                    return Err(BuildError::ResolverExtensionOnNonVirtualGraph {
                        id: extension.manifest.id.clone(),
                        directive: name.to_string(),
                        subgraph: self.ctx[self.ctx[id].subgraph_name_id].clone(),
                    });
                }
            };

            if let Some(other_id) =
                self.virtual_subgraph_to_selection_set_resolver[usize::from(id)].filter(|id| *id != extension.id)
            {
                return Err(
                    BuildError::SelectionSetResolverExtensionCannotBeMixedWithOtherResolvers(Box::new(
                        SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError {
                            id: extension.manifest.id.clone(),
                            subgraph: self.ctx[self.ctx[id].subgraph_name_id].clone(),
                            other_id: self.ctx[other_id].manifest.id.clone(),
                        },
                    )),
                );
            }
            self.virtual_subgraph_to_selection_set_resolver[usize::from(id)] = Some(extension.id);
        }

        let (argument_ids, requirements_record) = self
            .coerce_extension_directive_arguments(location, sdl, definition, arguments)
            .map_err(|err| {
                BuildError::ExtensionDirectiveArgumentsError(Box::new(ExtensionDirectiveArgumentsError {
                    location: location.to_string(self),
                    directive: name.to_string(),
                    id: extension.manifest.id.clone(),
                    err,
                }))
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
