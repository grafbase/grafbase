use std::str::FromStr as _;

use crate::extension::ExtensionDirectiveArgumentRecord;

use super::{BuildError, Context, ExtensionDirectiveRecord, ExtensionInputValueCoercer, GraphContext, SchemaLocation};

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
        federated_graph::ExtensionDirective {
            subgraph_id,
            extension_id,
            name,
            arguments,
        }: &federated_graph::ExtensionDirective,
    ) -> Result<ExtensionDirectiveRecord, BuildError> {
        let directive_name_id = self.get_or_insert_str(*name);
        let directive_name = &self.ctx.federated_graph[*name];

        let Some(sdl) = self[*extension_id].sdl.take() else {
            return Err(BuildError::MissingGraphQLDefinitions {
                id: self.get_extension_id(*extension_id),
                directive: directive_name.clone(),
            });
        };

        let Some(definition) = sdl.parsed.definitions().find_map(|def| match def {
            cynic_parser::type_system::Definition::Directive(dir) if dir.name() == directive_name => Some(dir),
            _ => None,
        }) else {
            return Err(BuildError::UnknownExtensionDirective {
                id: self.get_extension_id(*extension_id),
                directive: directive_name.to_string(),
            });
        };

        let start = self.graph.extension_directive_arguments.len();
        if let Some(arguments) = arguments {
            self.graph.extension_directive_arguments.reserve(arguments.len());
            let mut coercer = ExtensionInputValueCoercer {
                ctx: self,
                sdl: &sdl,
                current_injection_stage: Default::default(),
            };

            for (arg_name, value) in arguments {
                let name_id = coercer.ctx.get_or_insert_str(*arg_name);
                let name = &coercer.ctx.federated_graph[*arg_name];
                let Some(def) = definition.arguments().find(|arg| arg.name() == name) else {
                    return Err(BuildError::UnknownExtensionDirectiveArgument {
                        id: coercer.get_extension_id(*extension_id),
                        directive: directive_name.to_string(),
                        argument: name.to_string(),
                    });
                };
                let (value, injection_stage) = coercer.coerce_extension_value(def, value).map_err(|err| {
                    BuildError::ExtensionDirectiveArgumentsError {
                        location: location.to_string(coercer.ctx),
                        extension_id: Box::new(coercer.get_extension_id(*extension_id)),
                        err,
                    }
                })?;
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
        let argument_ids = (start..self.graph.extension_directive_arguments.len()).into();

        self[*extension_id].sdl = Some(sdl);

        Ok(ExtensionDirectiveRecord {
            subgraph_id: self.subgraphs[*subgraph_id],
            extension_id: self[*extension_id].catalog_id,
            name_id: directive_name_id,
            argument_ids,
        })
    }
}
