mod directive;
mod selection_set_resolvers;

use extension_catalog::{ExtensionCatalog, ExtensionId, Manifest};
use federated_graph::link::LinkDirective;
use rapidhash::RapidHashMap;
use std::str::FromStr as _;
use strum::IntoEnumIterator as _;

use cynic_parser_deser::ConstDeserializer;

use super::sdl::{ExtensionName, Sdl};

pub(crate) use directive::*;
pub(crate) use selection_set_resolvers::*;

#[derive(id_derives::IndexedFields)]
pub(crate) struct ExtensionsContext<'a> {
    map: RapidHashMap<ExtensionName<'a>, LoadedExtension<'a>>,
    pub composite_schema_extension_name: Option<ExtensionName<'a>>,
    pub catalog: &'a ExtensionCatalog,
}

const COMPOSITE_SCHEMA_URL: &str = "https://specs.grafbase.com/composite-schema/v1";
const GRAFBASE_SPEC_URL: &str = "https://specs.grafbase.com/grafbase";
const GRAFBASE_NAMEPSACE: &str = "grafbase";

pub(crate) struct LoadedExtension<'a> {
    pub id: ExtensionId,
    pub manifest: &'a Manifest,
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
    pub doc: cynic_parser::TypeSystemDocument,
    pub grafbase_scalars: Vec<(String, GrafbaseScalar)>,
}

impl<'a> ExtensionsContext<'a> {
    pub(super) async fn load<'sdl, 'ext>(sdl: &'sdl Sdl<'sdl>, catalog: &'ext ExtensionCatalog) -> Result<Self, String>
    where
        'sdl: 'a,
        'ext: 'a,
    {
        let mut extensions = Self {
            map: RapidHashMap::with_capacity_and_hasher(sdl.extensions.len(), Default::default()),
            composite_schema_extension_name: None,
            catalog,
        };
        for (name, extension) in &sdl.extensions {
            if extension.url.as_str() == COMPOSITE_SCHEMA_URL {
                extensions.composite_schema_extension_name = Some(*name);
                continue;
            }

            let manifest = extension_catalog::load_manifest(extension.url.clone())
                .await
                .map_err(|err| {
                    format!(
                        "Could not fetch extension manifest at '{}' for extensions '{}': {}",
                        extension.url, name, err
                    )
                })?;
            let Some(id) = catalog.find_compatible_extension(&manifest.id) else {
                return Err(format!("Extension {} was not installed", manifest.id));
            };
            let sdl = manifest
                .sdl
                .as_ref()
                .filter(|sdl| !sdl.trim().is_empty())
                .map(|sdl| cynic_parser::parse_type_system_document(sdl))
                .transpose()
                .map_err(|err| {
                    format!(
                        "For extension {}, failed to parse GraphQL definitions: {}",
                        manifest.id, err
                    )
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
                                format!(
                                    "For extension {}, failed to prase @link directive: {}",
                                    manifest.id, err
                                )
                            })?;
                            if !link.url.starts_with(GRAFBASE_SPEC_URL) {
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
                                    format!(
                                        "For extension {}, unsupported import '{}' from '{}'",
                                        manifest.id, name, GRAFBASE_SPEC_URL
                                    )
                                })?;
                                grafbase_scalars.push((alias.to_string(), scalar));
                            }
                        }
                    }
                    Ok(Some(ExtensionSdl {
                        doc: parsed,
                        grafbase_scalars,
                    }))
                })?;

            extensions.map.insert(
                *name,
                LoadedExtension {
                    id,
                    manifest: &catalog[id].manifest,
                    sdl,
                },
            );
        }

        Ok(extensions)
    }

    pub(super) fn get(&self, name: ExtensionName<'a>) -> LoadedExtensionOrCompositeSchema<'_, 'a> {
        match self.map.get(&name) {
            Some(extension) => LoadedExtensionOrCompositeSchema::Extension(extension),
            None => {
                if Some(name) == self.composite_schema_extension_name {
                    LoadedExtensionOrCompositeSchema::CompositeSchema
                } else {
                    unreachable!("Extension {name} not found, should have failed during ExtensionsContext creation.");
                }
            }
        }
    }
}

pub(crate) enum LoadedExtensionOrCompositeSchema<'a, 'sdl> {
    Extension(&'a LoadedExtension<'sdl>),
    CompositeSchema,
}
