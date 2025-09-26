mod directive;

use extension_catalog::{ExtensionCatalog, ExtensionId};
use rapidhash::fast::RapidHashMap;
use std::str::FromStr as _;
use strum::IntoEnumIterator as _;

use cynic_parser_deser::ConstDeserializer;

use crate::builder::sdl::{Import, LinkDirective, LinkUrl};

use super::sdl::{ExtensionName, LinkId, Sdl};

pub(crate) use directive::*;

#[derive(id_derives::IndexedFields)]
pub(crate) struct ExtensionsContext<'a> {
    pub catalog: &'a ExtensionCatalog,
    name_to_extension: RapidHashMap<ExtensionName<'a>, ExtensionId>,
    link_to_extension: RapidHashMap<LinkId, ExtensionId>,
    #[indexed_by(ExtensionId)]
    extension_sdl: Vec<Option<ParsedExtensionSdl>>,
}

const GRAFBASE_SPEC_URL: &str = "https://specs.grafbase.com/grafbase";
const GRAFBASE_NAMEPSACE: &str = "grafbase";

#[derive(Debug, strum_macros::EnumString, strum_macros::EnumIter, strum_macros::Display)]
pub(crate) enum GrafbaseScalar {
    InputValueSet,
    UrlTemplate,
    JsonTemplate,
    FieldSet,
}

pub(crate) struct ParsedExtensionSdl {
    pub doc: cynic_parser::TypeSystemDocument,
    pub grafbase_scalars: Vec<(String, GrafbaseScalar)>,
}

impl<'a> ExtensionsContext<'a> {
    pub(super) fn empty_with_catalog(catalog: &'a ExtensionCatalog) -> Self {
        Self {
            name_to_extension: RapidHashMap::default(),
            link_to_extension: RapidHashMap::default(),
            extension_sdl: {
                let mut v = Vec::with_capacity(catalog.len());
                v.resize_with(catalog.len(), || None);
                v
            },
            catalog,
        }
    }

    pub(super) async fn load<'sdl, 'ext>(
        sdl: &'sdl Sdl<'sdl>,
        catalog: &'ext ExtensionCatalog,
    ) -> Result<Self, Vec<super::Error>>
    where
        'sdl: 'a,
        'ext: 'a,
    {
        let mut extensions = Self {
            catalog,
            name_to_extension: RapidHashMap::with_capacity_and_hasher(sdl.extensions.len(), Default::default()),
            link_to_extension: Default::default(),
            extension_sdl: {
                let mut v = Vec::with_capacity(catalog.len());
                v.resize_with(catalog.len(), || None);
                v
            },
        };
        let mut errors = Vec::new();

        for (name, extension) in &sdl.extensions {
            let mut extension_id = if extension.url.scheme() == "file"
                && let Ok(mut path) = extension.url.to_file_path()
            {
                if !path.ends_with("manifest.json") {
                    path.push("manifest.json");
                }
                std::fs::read(path)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<extension_catalog::VersionedManifest>(&bytes).ok())
                    .map(|manifest| manifest.into_latest())
                    .and_then(|manifest| {
                        catalog
                            .iter_with_id()
                            .find(|(_, ext)| ext.manifest.id == manifest.id)
                            .map(|(id, _)| id)
                    })
            } else {
                None
            };

            extension_id = extension_id.or_else(|| {
                let url = LinkUrl::from(extension.url.as_str());
                catalog.find_compatible_extension(url.as_str(), url.name.as_deref(), url.version.as_ref())
            });

            let Some(extension_id) = extension_id else {
                errors.push(super::Error::new(format!(
                    "Could not find a matching extension for {}",
                    extension.url
                )));
                continue;
            };
            extensions.name_to_extension.insert(*name, extension_id);

            if extensions[extension_id].is_none()
                && let Some(extension_sdl) = parse_manifest_sdl(catalog, extension_id, &mut errors)
            {
                extensions[extension_id] = Some(extension_sdl);
            }
        }

        for (link_id, link) in sdl.iter_links() {
            let Some(extension_id) = catalog.find_compatible_extension(
                link.url.as_str(),
                link.url.name.as_deref(),
                link.url.version.as_ref(),
            ) else {
                continue;
            };
            tracing::debug!(
                "Matched link URL {} to extension {}",
                link.url.as_str(),
                catalog[extension_id].manifest.id
            );
            extensions.link_to_extension.insert(link_id, extension_id);

            if extensions[extension_id].is_none()
                && let Some(extension_sdl) = parse_manifest_sdl(catalog, extension_id, &mut errors)
            {
                extensions[extension_id] = Some(extension_sdl);
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(extensions)
        }
    }

    pub(super) fn get_by_name(&self, name: ExtensionName<'_>) -> LoadedExtension<'_> {
        match self.name_to_extension.get(&name) {
            Some(id) => LoadedExtension {
                id: *id,
                manifest: &self.catalog[*id].manifest,
                sdl: self[*id].as_ref(),
            },
            None => {
                unreachable!("Extension {name} not found, should have failed during ExtensionsContext creation.");
            }
        }
    }

    pub(super) fn get_by_link_id(&self, id: LinkId) -> Option<LoadedExtension<'_>> {
        self.link_to_extension.get(&id).map(|id| LoadedExtension {
            id: *id,
            manifest: &self.catalog[*id].manifest,
            sdl: self[*id].as_ref(),
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LoadedExtension<'a> {
    pub id: ExtensionId,
    pub manifest: &'a extension_catalog::Manifest,
    pub sdl: Option<&'a ParsedExtensionSdl>,
}

fn parse_manifest_sdl(
    catalog: &ExtensionCatalog,
    extension_id: ExtensionId,
    errors: &mut Vec<super::Error>,
) -> Option<ParsedExtensionSdl> {
    let manifest = &catalog[extension_id].manifest;
    let sdl_str = manifest.sdl.as_ref().filter(|sdl| !sdl.trim().is_empty())?;

    let parsed = match cynic_parser::parse_type_system_document(sdl_str) {
        Ok(parsed) => parsed,
        Err(err) => {
            errors.push(super::Error::new(format!(
                "For extension {}, failed to parse GraphQL definitions: {}",
                manifest.id, err
            )));
            return None;
        }
    };

    let mut grafbase_scalars = Vec::new();
    let mut had_error = false;

    for definition in parsed.definitions() {
        let cynic_parser::type_system::Definition::SchemaExtension(ext) = definition else {
            continue;
        };
        for dir in ext.directives() {
            if dir.name() != "link" {
                continue;
            }
            let link = match dir.deserialize::<LinkDirective>() {
                Ok(link) => link,
                Err(err) => {
                    errors.push(super::Error::new(format!(
                        "For extension {}, failed to parse @link directive: {}",
                        manifest.id, err
                    )));
                    had_error = true;
                    continue;
                }
            };
            if !link.url.as_str().starts_with(GRAFBASE_SPEC_URL) {
                continue;
            }
            let namespace = link.r#as.unwrap_or(GRAFBASE_NAMEPSACE);
            grafbase_scalars.extend(GrafbaseScalar::iter().map(|s| (format!("{namespace}__{s}"), s)));
            for import in link.import.unwrap_or_default() {
                let (name, alias) = match import {
                    Import::String(name) => (name, name),
                    Import::Qualified(q) => (q.name, q.r#as.unwrap_or(q.name)),
                };
                let scalar = match GrafbaseScalar::from_str(name) {
                    Ok(scalar) => scalar,
                    Err(_) => {
                        errors.push(super::Error::new(format!(
                            "For extension {}, unsupported import '{}' from '{}'",
                            manifest.id, name, GRAFBASE_SPEC_URL
                        )));
                        had_error = true;
                        continue;
                    }
                };
                grafbase_scalars.push((alias.to_string(), scalar));
            }
        }
    }

    if had_error {
        return None;
    }

    Some(ParsedExtensionSdl {
        doc: parsed,
        grafbase_scalars,
    })
}
