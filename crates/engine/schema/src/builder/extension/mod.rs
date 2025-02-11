use std::str::FromStr as _;

use crate::extension::ExtensionDirectiveArgumentRecord;

use super::{
    BuildError, Context, ExtensionDirectiveArgumentsError, ExtensionDirectiveId, ExtensionDirectiveLocationError,
    ExtensionDirectiveRecord, ExtensionInputValueCoercer, GraphContext, InputValueError, SchemaLocation,
};

pub(crate) struct SchemaExtension {
    pub catalog_id: extension_catalog::ExtensionId,
    pub sdl: Option<ExtensionSdl>,
}

pub(crate) struct ExtensionSdl {
    pub parsed: cynic_parser::TypeSystemDocument,
}

impl Context<'_> {
    pub(super) async fn load_extension_links(&mut self) -> Result<(), BuildError> {
        for extension in &self.federated_graph.extensions {
            let url_str = &self.federated_graph[extension.url];
            let url =
                url::Url::from_str(&self.federated_graph[extension.url]).map_err(|err| BuildError::InvalidUrl {
                    url: url_str.to_string(),
                    err: err.to_string(),
                })?;
            let manifest =
                extension_catalog::load_manifest(url)
                    .await
                    .map_err(|err| BuildError::CouldNotLoadExtension {
                        url: url_str.to_string(),
                        err: err.to_string(),
                    })?;
            let Some(catalog_id) = self.extension_catalog.find_compatible_extension(&manifest.id) else {
                return Err(BuildError::UnsupportedExtension {
                    id: manifest.id.clone(),
                });
            };
            let sdl = manifest
                .sdl
                .as_ref()
                .map(|sdl| cynic_parser::parse_type_system_document(sdl))
                .transpose()
                .map_err(|err| BuildError::CouldNotParseExtension {
                    id: manifest.id.clone(),
                    err: err.to_string(),
                })?
                .map(|parsed| ExtensionSdl { parsed });

            self.extensions.push(SchemaExtension { catalog_id, sdl });
        }

        Ok(())
    }

    fn get_extension_id(&self, id: federated_graph::ExtensionId) -> extension_catalog::Id {
        self.extension_catalog[self[id].catalog_id].manifest.id.clone()
    }
}

impl GraphContext<'_> {
    pub(crate) fn ingest_extension_directive(
        &mut self,
        location: SchemaLocation,
        subgraph_id: federated_graph::SubgraphId,
        extension_id: federated_graph::ExtensionId,
        name: federated_graph::StringId,
        arguments: &Option<Vec<(federated_graph::StringId, federated_graph::Value)>>,
    ) -> Result<ExtensionDirectiveId, BuildError> {
        let directive_name_id = self.get_or_insert_str(name);
        let directive_name = &self.ctx.federated_graph[name];

        let Some(sdl) = self[extension_id].sdl.take() else {
            return Err(BuildError::MissingGraphQLDefinitions {
                id: self.get_extension_id(extension_id),
                directive: directive_name.clone(),
            });
        };

        let Some(definition) = sdl.parsed.definitions().find_map(|def| match def {
            cynic_parser::type_system::Definition::Directive(dir) if dir.name() == directive_name => Some(dir),
            _ => None,
        }) else {
            return Err(BuildError::UnknownExtensionDirective {
                id: self.get_extension_id(extension_id),
                directive: directive_name.to_string(),
            });
        };

        let cynic_location = location.to_cynic_location();
        if definition
            .locations()
            .all(|loc| loc.as_str() != cynic_location.as_str())
        {
            return Err(BuildError::ExtensionDirectiveLocationError(Box::new(
                ExtensionDirectiveLocationError {
                    id: self.get_extension_id(extension_id),
                    directive: directive_name.to_string(),
                    location: cynic_location.as_str(),
                    expected: definition.locations().map(|loc| loc.as_str()).collect(),
                },
            )));
        }

        let federated_graph = self.ctx.federated_graph;
        let start = self.graph.extension_directive_arguments.len();
        if let Some(arguments) = arguments {
            let mut arguments = arguments.iter().collect::<Vec<_>>();
            self.graph.extension_directive_arguments.reserve(arguments.len());
            let mut coercer = ExtensionInputValueCoercer {
                ctx: self,
                sdl: &sdl,
                current_injection_stage: Default::default(),
            };

            for def in definition.arguments() {
                let name_id = coercer.ctx.strings.get_or_new(def.name());
                let sdl_value = arguments
                    .iter()
                    .position(|(name, _)| federated_graph[*name] == def.name())
                    .map(|ix| &arguments.swap_remove(ix).1);

                let maybe_coerced_argument = coercer.coerce_argument(def, sdl_value).map_err(|err| {
                    BuildError::ExtensionDirectiveArgumentsError(Box::new(ExtensionDirectiveArgumentsError {
                        location: location.to_string(coercer.ctx),
                        directive: directive_name.to_string(),
                        extension_id: coercer.get_extension_id(extension_id),
                        err,
                    }))
                })?;

                if let Some((value, injection_stage)) = maybe_coerced_argument {
                    coercer
                        .ctx
                        .graph
                        .extension_directive_arguments
                        .push(ExtensionDirectiveArgumentRecord {
                            name_id,
                            value,
                            injection_stage,
                        });
                }
            }

            if let Some((name, _)) = arguments.first() {
                return Err(BuildError::ExtensionDirectiveArgumentsError(Box::new(
                    ExtensionDirectiveArgumentsError {
                        location: location.to_string(coercer.ctx),
                        directive: directive_name.to_string(),
                        extension_id: coercer.get_extension_id(extension_id),
                        err: InputValueError::UnknownArgument(federated_graph[*name].clone()).into(),
                    },
                )));
            }
        }

        let argument_ids = (start..self.graph.extension_directive_arguments.len()).into();

        self[extension_id].sdl = Some(sdl);

        let record = ExtensionDirectiveRecord {
            subgraph_id: self.subgraphs[subgraph_id],
            extension_id: self[extension_id].catalog_id,
            name_id: directive_name_id,
            argument_ids,
        };
        self.graph.extension_directives.push(record);
        let id = (self.graph.extension_directives.len() - 1).into();
        Ok(id)
    }
}
