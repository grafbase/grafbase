use std::str::FromStr as _;
use strum::IntoEnumIterator as _;

use cynic_parser_deser::ConstDeserializer;
use federated_graph::link::LinkDirective;

use super::{
    BuildError, Context, ExtensionDirectiveArgumentsError, ExtensionDirectiveId, ExtensionDirectiveLocationError,
    ExtensionDirectiveRecord, GraphContext, SchemaLocation,
};

const GRAFBASE_SPEC_URL: &str = "https://specs.grafbase.com/grafbase";
const GRAFBASE_NAMEPSACE: &str = "grafbase";

pub(crate) struct SchemaExtension {
    pub catalog_id: extension_catalog::ExtensionId,
    pub sdl: Option<ExtensionSdl>,
}

#[derive(Debug, strum_macros::EnumString, strum_macros::EnumIter, strum_macros::Display)]
pub(crate) enum GrafbaseScalar {
    InputValueSet,
    UrlTemplate,
    JsonTemplate,
    FieldSet,
}

pub(crate) struct ExtensionSdl {
    pub parsed: cynic_parser::TypeSystemDocument,
    pub grafbase_scalars: Vec<(String, GrafbaseScalar)>,
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
                })
                .and_then(|parsed| {
                    let Some(parsed) = parsed else {
                        return Ok(None);
                    };
                    let mut grafbase_scalars = Vec::new();
                    for definition in parsed.definitions() {
                        let cynic_parser::type_system::Definition::SchemaExtension(ext) = definition else {
                            continue;
                        };
                        for dir in ext.directives() {
                            if dir.name() != "link" {
                                continue;
                            }
                            let link = dir.deserialize::<LinkDirective>().map_err(|err| {
                                BuildError::ExtensionCouldNotReadLink {
                                    id: manifest.id.clone(),
                                    err: err.to_string(),
                                }
                            })?;
                            if link.url != GRAFBASE_SPEC_URL {
                                continue;
                            }
                            let namespace = link.r#as.unwrap_or(GRAFBASE_NAMEPSACE);
                            grafbase_scalars.extend(GrafbaseScalar::iter().map(|s| (format!("{namespace}__{s}"), s)));
                            for import in link.import.unwrap_or_default() {
                                let (name, alias) = match import {
                                    federated_graph::link::Import::String(name) => (name, name),
                                    federated_graph::link::Import::Qualified(q) => (q.name, q.r#as.unwrap_or(q.name)),
                                };
                                let scalar = GrafbaseScalar::from_str(name).map_err(|_| {
                                    BuildError::ExtensionLinksToUnknownGrafbaseDefinition {
                                        id: manifest.id.clone(),
                                        name: name.to_string(),
                                    }
                                })?;
                                grafbase_scalars.push((alias.to_string(), scalar));
                            }
                        }
                    }
                    Ok(Some(ExtensionSdl {
                        parsed,
                        grafbase_scalars,
                    }))
                })?;

            self.extensions.push(SchemaExtension { catalog_id, sdl })
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

        let kind = self
            .ctx
            .extension_catalog
            .get_directive_kind(self[extension_id].catalog_id, directive_name);

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
        let (argument_ids, requirements_record) = self
            .coerce_extension_directive_arguments(location, &sdl, definition, arguments)
            .map_err(|err| {
                BuildError::ExtensionDirectiveArgumentsError(Box::new(ExtensionDirectiveArgumentsError {
                    location: location.to_string(self),
                    directive: directive_name.to_string(),
                    extension_id: self.get_extension_id(extension_id),
                    err,
                }))
            })?;

        self[extension_id].sdl = Some(sdl);

        let record = ExtensionDirectiveRecord {
            subgraph_id: self.subgraphs[subgraph_id],
            extension_id: self[extension_id].catalog_id,
            name_id: directive_name_id,
            kind,
            argument_ids,
            requirements_record,
        };
        self.graph.extension_directives.push(record);
        let id = (self.graph.extension_directives.len() - 1).into();
        Ok(id)
    }
}
