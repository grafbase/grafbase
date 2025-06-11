use schema::Schema;

use crate::assert_solving_snapshots;

#[tokio::test]
async fn direct_lookup_call() {
    let tmpdir = tempfile::tempdir().unwrap();
    let manifest = extension_catalog::Manifest {
        id: "ext-1.0.0".parse().unwrap(),
        r#type: extension_catalog::Type::SelectionSetResolver(Default::default()),
        sdk_version: "0.0.0".parse().unwrap(),
        minimum_gateway_version: "0.0.0".parse().unwrap(),
        description: String::new(),
        sdl: Some(
            r#"
            directive @init on SCHEMA
            "#
            .into(),
        ),
        readme: None,
        homepage_url: None,
        repository_url: None,
        license: None,
        permissions: Default::default(),
        event_filter: Default::default(),
    };

    std::fs::write(
        tmpdir.path().join("manifest.json"),
        serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
    )
    .unwrap();

    let mut catalog = extension_catalog::ExtensionCatalog::default();
    let wasm_path = tmpdir.path().join("extension.wasm");
    std::fs::write(&wasm_path, b"wasm").unwrap();
    catalog.push(extension_catalog::Extension { manifest, wasm_path });

    let sdl = format!(
        r#"
        enum join__Graph {{
            PG @join__graph(name: "pg")
        }}

        enum extension__Link
        {{
          EXT @extension__link(url: "{}", schemaDirectives: [{{graph: PG, name: "init", arguments: {{}}}}])
        }}

        type Query @join__type(graph: PG) {{
            userLookup(id: [ID!]): [User!] @composite__lookup(graph: PG)
        }}

        type User @join__type(graph: PG, key: "id", resolvable: false) {{
            id: ID!
            name: String!
        }}
        "#,
        url::Url::from_file_path(tmpdir.path()).unwrap()
    );

    let schema = Schema::builder(&sdl).extensions(None, &catalog).build().await.unwrap();

    assert_solving_snapshots!(
        "direct_lookup_call",
        schema,
        r#"
        query {
            userLookup(id: ["1"]) {
                id
                name
            }
        }
        "#
    );
}
