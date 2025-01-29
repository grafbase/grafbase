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

    let (name, version) = parse_link_url(url)
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

                subgraphs.push_linked_definition(subgraphs::LinkedDefinitionRecord {
                    linked_schema_id,
                    original_name,
                    imported_as: None,
                });
            }
            graphql_federated_graph::link::Import::Qualified(graphql_federated_graph::link::QualifiedImport {
                name,
                r#as,
            }) => {
                let name = name.trim_start_matches("@");
                let original_name = subgraphs.strings.intern(name);
                let imported_as = r#as.map(|r#as| subgraphs.strings.intern(r#as));

                subgraphs.push_linked_definition(subgraphs::LinkedDefinitionRecord {
                    linked_schema_id,
                    original_name,
                    imported_as,
                });
            }
        }
    }
}

struct LinkUrl {
    #[expect(unused)]
    url: url::Url,
    name: Option<String>,
    version: Option<String>,
}

/// https://specs.apollo.dev/link/v1.0/#@link.url
fn parse_link_url(url: &str) -> Option<LinkUrl> {
    // Must be a url, or treated as an opaque identifier (which is valid).
    let url = url::Url::parse(url).ok()?;

    let segments = url.path_segments()?;

    let mut reversed_segments = segments.rev();

    let Some(maybe_version_or_name) = reversed_segments.next() else {
        return Some(LinkUrl {
            url,
            name: None,
            version: None,
        });
    };

    if is_valid_version(maybe_version_or_name) {
        let name = reversed_segments
            .next()
            .filter(|s| is_valid_graphql_name(s))
            .map(String::from);

        let version = Some(maybe_version_or_name.to_owned());

        Some(LinkUrl { url, name, version })
    } else if is_valid_graphql_name(maybe_version_or_name) {
        let name = Some(maybe_version_or_name.to_owned());

        Some(LinkUrl {
            url,
            name,
            version: None,
        })
    } else {
        Some(LinkUrl {
            url,
            name: None,
            version: None,
        })
    }
}

fn is_valid_version(s: &str) -> bool {
    let mut chars = s.chars();

    let Some('v') = chars.next() else { return false };

    let Some(digit) = chars.next() else { return false };

    if !digit.is_ascii_digit() {
        return false;
    };

    chars.all(|char| char.is_ascii_digit() || char == '.')
}

fn is_valid_graphql_name(s: &str) -> bool {
    let mut chars = s.chars();

    let Some(first_char) = chars.next() else {
        return false;
    };

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}
