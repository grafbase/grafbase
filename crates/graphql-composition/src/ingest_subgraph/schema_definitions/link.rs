use super::*;
use crate::federated_graph::{Import, LinkDirectiveDeserialize, QualifiedImport};
use cynic_parser_deser::ConstDeserializer;

fn is_grafbase_extension_registry_url(url: &url::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }

    if let Some(host) = url.host_str() {
        if host == "extensions.grafbase.com" {
            return true;
        }
        if host == "grafbase.com" && url.path().starts_with("/extensions") {
            return true;
        }
    }

    false
}

pub(super) fn ingest_link_directive(directive: ast::Directive<'_>, subgraph_id: SubgraphId, subgraphs: &mut Subgraphs) {
    let LinkDirectiveDeserialize {
        url,
        r#as,
        import,
        r#for: _,
    }: LinkDirectiveDeserialize<'_> = match directive.deserialize() {
        Ok(directive) => directive,
        Err(err) => {
            subgraphs.push_ingestion_diagnostic(subgraph_id, format!("Invalid `@link` directive: {err}"));
            return;
        }
    };

    let link_url = subgraphs::parse_link_url(url);

    let name = link_url
        .as_ref()
        .and_then(|link_url| link_url.name.as_deref())
        .map(|name| subgraphs.strings.intern(name));

    let linked_schema_type = if let Some(federation_spec) = subgraphs::FederationSpec::from_url(url) {
        subgraphs::LinkedSchemaType::FederationSpec(federation_spec)
    } else {
        subgraphs::LinkedSchemaType::Other
    };

    let url = subgraphs.strings.intern(url);
    let r#as = r#as.map(|r#as| subgraphs.strings.intern(r#as));

    if let Some(link_url) = link_url.as_ref() {
        ingest_grafbase_extension_from_link(subgraphs, url, r#as, link_url);
    }

    let linked_schema_id = subgraphs.push_linked_schema(subgraphs::LinkedSchemaRecord {
        subgraph_id,
        linked_schema_type,
        url,
        r#as,
        name_from_url: name,
    });

    for import in import.into_iter().flatten() {
        match import {
            Import::String(name) => {
                let name = name.trim_start_matches("@");
                let original_name = subgraphs.strings.intern(name);

                subgraphs.push_linked_definition(
                    subgraph_id,
                    subgraphs::LinkedDefinitionRecord {
                        linked_schema_id,
                        original_name,
                        imported_as: None,
                    },
                );
            }
            Import::Qualified(QualifiedImport { name, r#as }) => {
                let is_directive = name.starts_with('@');

                let trimmed_name = name.trim_start_matches("@");
                let original_name = subgraphs.strings.intern(trimmed_name);

                let imported_as = if let Some(r#as) = r#as {
                    if r#as.starts_with('@') != is_directive {
                        if is_directive {
                            subgraphs.push_ingestion_diagnostic(
                                subgraph_id,
                                format!("Error in @link import: `{name}` is a directive, but it is imported as `{as}`. Missing @ prefix."),
                            );
                        } else if !is_directive {
                            subgraphs.push_ingestion_diagnostic(
                                subgraph_id,
                                format!("Error in @link import: `{name}` is not a directive, but it is imported as `{as}`. Consider removing the @ prefix."),
                            );
                        }
                    }

                    Some(subgraphs.strings.intern(r#as.trim_start_matches("@")))
                } else {
                    None
                };

                subgraphs.push_linked_definition(
                    subgraph_id,
                    subgraphs::LinkedDefinitionRecord {
                        linked_schema_id,
                        original_name,
                        imported_as,
                    },
                );
            }
        }
    }
}

/// Treat `@link`ed schemas with a file url or Grafbase extension registry URLs as extensions.
fn ingest_grafbase_extension_from_link(
    subgraphs: &mut Subgraphs,
    url: subgraphs::StringId,
    r#as: Option<subgraphs::StringId>,
    link_url: &subgraphs::LinkUrl,
) {
    if link_url.url.scheme() == "file" {
        let Some(name) = r#as.or_else(|| {
            link_url.url.to_file_path().ok().and_then(|path| {
                let file_name = path.file_name()?.to_str()?;
                match file_name {
                    // This is going to be the name for locally built extension. In that case, the parent directory has a more descriptive name.
                    "build" => path
                        .parent()
                        .and_then(|parent| parent.file_name()?.to_str())
                        .map(|s| subgraphs.strings.intern(s)),
                    _ => Some(subgraphs.strings.intern(file_name)),
                }
            })
        }) else {
            return;
        };

        if subgraphs.extension_is_defined(name) {
            return;
        }

        subgraphs.push_extension(subgraphs::ExtensionRecord {
            url,
            link_url: url,
            name,
        });
    }

    if is_grafbase_extension_registry_url(&link_url.url) {
        let Some(name) = r#as.or_else(|| {
            let mut segments = link_url.url.path_segments()?;

            segments.next_back()?;

            segments.next_back().map(|s| subgraphs.strings.intern(s))
        }) else {
            return;
        };

        subgraphs.push_extension(subgraphs::ExtensionRecord {
            url,
            link_url: url,
            name,
        });
    }
}
