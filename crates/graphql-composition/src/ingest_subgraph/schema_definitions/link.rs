use super::*;
use cynic_parser_deser::ConstDeserializer;

pub(super) fn ingest_link_directive(directive: ast::Directive<'_>, subgraph_id: SubgraphId, subgraphs: &mut Subgraphs) {
    let graphql_federated_graph::link::LinkDirective {
        url,
        r#as,
        import,
        r#for: _,
    }: graphql_federated_graph::link::LinkDirective<'_> = match directive.deserialize() {
        Ok(directive) => directive,
        Err(err) => {
            subgraphs.push_ingestion_diagnostic(subgraph_id, format!("Invalid `@link` directive: {err}"));
            return;
        }
    };

    let (name, version) = subgraphs::parse_link_url(url)
        .map(|url| (url.name, url.version))
        .unwrap_or_default();

    let name = name.map(|name| subgraphs.strings.intern(name));
    let version = version.map(|version| subgraphs.strings.intern(version));

    let url = subgraphs.strings.intern(url);
    let r#as = r#as.map(|r#as| subgraphs.strings.intern(r#as));

    let linked_schema_id = subgraphs.push_linked_schema(subgraphs::LinkedSchemaRecord {
        subgraph_id,
        url,
        r#as,
        name_from_url: name,
        version_from_url: version,
    });

    for import in import.into_iter().flatten() {
        match import {
            graphql_federated_graph::link::Import::String(name) => {
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
            graphql_federated_graph::link::Import::Qualified(graphql_federated_graph::link::QualifiedImport {
                name,
                r#as,
            }) => {
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
