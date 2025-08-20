use cynic_parser::{TypeSystemDocument, type_system as ast};
use extension_catalog::load_manifest;
use futures::{TryFutureExt as _, future::join_all};
use gateway_config::Config;
use graphql_composition::LoadedExtension;
use url::Url;

pub(super) async fn detect_extensions(config: &Config, parsed_schema: &TypeSystemDocument) -> Vec<LoadedExtension> {
    let link_directives = parsed_schema
        .definitions()
        .filter_map(|definition| match definition {
            ast::Definition::Schema(schema_definition) | ast::Definition::SchemaExtension(schema_definition) => {
                Some(schema_definition.directives())
            }
            _ => None,
        })
        .flatten()
        .filter(|directive| directive.name() == "link");

    let mut urls = link_directives
        .into_iter()
        .filter_map(|link_directive| {
            link_directive
                .argument("url")
                .and_then(|value| value.value().as_str())
                .and_then(|link_url| {
                    let url = if let Some(url) = link_url.strip_prefix("./") {
                        config
                            .parent_dir_path()
                            .map(|p| p.join(url))
                            .and_then(|p| Url::from_file_path(p).ok())
                    } else {
                        link_url.parse::<Url>().ok()
                    };
                    url.map(|url| (link_url, url))
                })
        })
        // These are for sure not grafbase extensions.
        .filter(|(_, url)| url.domain() != Some("specs.apollo.dev") && url.domain() != Some("specs.grafbase.com"))
        .collect::<Vec<_>>();

    urls.sort_unstable();
    urls.dedup();

    let futures = urls
        .into_iter()
        .map(|(link_url, url)| load_manifest(url.clone()).map_ok(move |manifest| (link_url, url, manifest)));

    let extensions = join_all(futures)
        .await
        .into_iter()
        .filter_map(|result| result.ok())
        .map(|(link_url, url, manifest)| LoadedExtension {
            link_url: link_url.to_owned(),
            url,
            name: manifest.id.name,
        })
        .collect();

    tracing::info!(?extensions, "Detected extensions");

    extensions
}

#[cfg(test)]
mod tests {
    use super::*;
    use cynic_parser::parse_type_system_document;
    use std::fs;

    #[tokio::test]
    async fn detect_extensions_with_only_federation() {
        let schema = r#"
           extend schema @link(url: "https://specs.apollo.dev/federation/v2.7")

           type Query {
                hi: String
           }

           enum ExtendedBoolean {
                TRUE
                FALSE
                UNCLEAR
           }
        "#;

        let ast = parse_type_system_document(schema).unwrap();

        let extensions = detect_extensions(&Default::default(), &ast).await;

        assert!(extensions.is_empty(), "Expected empty, got {extensions:#?}");
    }

    #[tokio::test]
    async fn detect_extensions_unrelated_file_and_federation() {
        let schema = r#"
           extend schema @link(url: "https://specs.apollo.dev/federation/v2.7") @link(url: "file:Cargo.toml")

           type Query {
                hi: String
           }

           enum ExtendedBoolean {
                TRUE
                FALSE
                UNCLEAR
           }
        "#;

        let ast = parse_type_system_document(schema).unwrap();

        let extensions = detect_extensions(&Default::default(), &ast).await;

        assert!(extensions.is_empty(), "Expected empty, got {extensions:#?}");
    }

    #[tokio::test]
    async fn detect_extensions_with_link_to_manifest() {
        let manifest = extension::Manifest {
            id: extension::Id {
                name: "test-extension".to_owned(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension::Type::FieldResolver(extension::FieldResolverType {
                resolver_directives: Some(vec!["@test".to_owned()]),
            }),
            sdk_version: "1.0.0".parse().unwrap(),
            minimum_gateway_version: "1.0.0".parse().unwrap(),
            sdl: None,
            description: "An extension in a test".to_owned(),
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            readme: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        }
        .into_versioned();

        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");
        let file = fs::File::create(&manifest_path).unwrap();
        serde_json::to_writer(file, &manifest).unwrap();

        let schema = format!(
            r###"
            extend schema @link(url: "https://specs.apollo.dev/federation/v2.7") @link(url: "file://{}")

            type Query {{
                 hi: String
            }}

            enum ExtendedBoolean {{
                 TRUE
                 FALSE
                 UNCLEAR
            }}
            "###,
            temp_dir.path().display().to_string().replace('\\', r#"\\"#)
        );

        eprintln!("{schema}");
        let ast = parse_type_system_document(&schema).unwrap();

        let detected_extensions = detect_extensions(&Default::default(), &ast).await;

        assert_eq!(detected_extensions.len(), 1);
        assert_eq!(detected_extensions[0].name, "test-extension");
    }
}
