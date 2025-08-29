use super::*;
use crate::federated_graph::{Import, LinkDirective, QualifiedImport};
use cynic_parser_deser::ConstDeserializer;

pub(super) fn ingest_link_directive(directive: ast::Directive<'_>, subgraph_id: SubgraphId, subgraphs: &mut Subgraphs) {
    let LinkDirective {
        url,
        r#as,
        import,
        r#for: _,
    }: LinkDirective<'_> = match directive.deserialize() {
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

    // Treat `@link`ed schemas with a file url as extensions.
    if let Some(name) = r#as
        && let Some(link_url) = &link_url
        && link_url.url.scheme() == "file"
        && !subgraphs.extension_is_defined(name)
    {
        subgraphs.push_extension(subgraphs::ExtensionRecord {
            url,
            link_url: url,
            name,
        });
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
