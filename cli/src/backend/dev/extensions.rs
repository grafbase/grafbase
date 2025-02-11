use cynic_parser::{type_system as ast, TypeSystemDocument};
use extension_catalog::load_manifest;
use futures::{future::join_all, TryFutureExt as _};
use url::Url;

#[derive(Debug)]
pub(super) struct DetectedExtension {
    pub(super) url: String,
    pub(super) name: String,
}

pub(super) async fn detect_extensions(parsed_schema: &TypeSystemDocument) -> Vec<DetectedExtension> {
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

    let urls = link_directives
        .into_iter()
        .filter_map(|link_directive| {
            link_directive
                .argument("url")
                .and_then(|value| value.value().as_str())
                .and_then(|url| url.parse().ok())
        })
        // These are for sure not grafbase extensions.
        .filter(|url: &Url| url.domain() != Some("specs.apollo.dev"));

    let futures = urls.map(|url| load_manifest(url.clone()).map_ok(move |manifest| (url, manifest)));

    let extensions = join_all(futures)
        .await
        .into_iter()
        .filter_map(|result| result.ok())
        .map(|(url, manifest)| DetectedExtension {
            url: url.to_string(),
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

        let extensions = detect_extensions(&ast).await;

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

        let extensions = detect_extensions(&ast).await;

        assert!(extensions.is_empty(), "Expected empty, got {extensions:#?}");
    }

    #[tokio::test]
    async fn detect_extensions_with_link_to_manifest() {
        let manifest = extension::Manifest {
            id: extension::Id {
                name: "test-extension".to_owned(),
                version: "1.0.0".parse().unwrap(),
            },
            kind: extension::Kind::FieldResolver(extension::FieldResolver {
                resolver_directives: vec!["@test".to_owned()],
            }),
            sdk_version: "1.0.0".parse().unwrap(),
            minimum_gateway_version: "1.0.0".parse().unwrap(),
            sdl: None,
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

        let detected_extensions = detect_extensions(&ast).await;

        assert_eq!(detected_extensions.len(), 1);
        assert_eq!(detected_extensions[0].name, "test-extension");
    }
}
