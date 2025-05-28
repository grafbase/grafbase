use duct::cmd;
use extension::VersionedManifest;
use tempfile::tempdir;

use crate::{cargo_bin, extension::use_latest_grafbase_sdk_in_cargo_toml};

#[test]
fn init() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", "--type", "authorization", &*project_path_str];

    let result = cmd(cargo_bin("grafbase"), &args)
        .unchecked()
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );

    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();

    insta::assert_snapshot!(&cargo_toml, @r#"
    [package]
    name = "test-project"
    version = "0.1.0"
    edition = "2024"
    license = "Apache-2.0"

    [lib]
    crate-type = ["cdylib"]

    [profile.release]
    opt-level = "z"
    strip = true
    lto = true
    codegen-units = 1

    [dependencies]
    grafbase-sdk = "0.16.0"
    serde = { version = "1", features = ["derive"] }

    [dev-dependencies]
    indoc = "2"
    insta = { version = "1", features = ["json"] }
    grafbase-sdk = { version = "0.16.0", features = ["test-utils"] }
    tokio = { version = "1", features = ["rt-multi-thread", "macros", "test-util"] }
    serde_json = "1"
    "#);

    let extension_toml = std::fs::read_to_string(project_path.join("extension.toml")).unwrap();

    insta::assert_snapshot!(&extension_toml, @r##"
    [extension]
    name = "test-project"
    version = "0.1.0"
    type = "authorization"
    description = "A new extension"
    # homepage_url = "https://example.com/my-extension"
    # repository_url = "https://github.com/my-username/my-extension"
    # license = "MIT"

    # These are the default permissions for the extension.
    # The user can enable or disable them as needed in the gateway
    # configuration file.
    [permissions]
    network = false
    stdout = false
    stderr = false
    environment_variables = false
    "##);

    let lib_rs = std::fs::read_to_string(project_path.join("src/lib.rs")).unwrap();

    insta::assert_snapshot!(&lib_rs, @r##"
    use grafbase_sdk::{
        AuthorizationExtension, IntoQueryAuthorization,
        types::{AuthorizationDecisions, Configuration, Error, ErrorResponse, QueryElements, SubgraphHeaders, Token},
    };

    #[derive(AuthorizationExtension)]
    struct TestProject;

    impl AuthorizationExtension for TestProject {
        fn new(config: Configuration) -> Result<Self, Error> {
            Ok(Self)
        }

        fn authorize_query(
            &mut self,
            headers: &mut SubgraphHeaders,
            token: Token,
            elements: QueryElements<'_>,
        ) -> Result<impl IntoQueryAuthorization, ErrorResponse> {
            Ok(AuthorizationDecisions::deny_all("Not authorized"))
        }
    }
    "##);

    let definitions = std::fs::read_to_string(project_path.join("definitions.graphql")).unwrap();

    insta::assert_snapshot!(&definitions, @r#"
    """
    Fill in here the directives and types that the extension needs.
    """
    directive @testProjectConfiguration(arg1: String) repeatable on SCHEMA
    directive @testProjectDirective on FIELD_DEFINITION
    "#);

    let tests_rs = std::fs::read_to_string(project_path.join("tests/integration_tests.rs")).unwrap();

    insta::assert_snapshot!(&tests_rs, @r##"
    use grafbase_sdk::test::{DynamicSchema, TestConfig, TestRunner};
    use indoc::indoc;

    #[tokio::test]
    async fn test_example() {
        // Run the tests with `cargo test`.

        // Create a subgraph with a single field
        let subgraph = DynamicSchema::builder(r#"type Query { hi: String }"#)
            .with_resolver("Query", "hi", String::from("hello"))
            .into_subgraph("test")
            .unwrap();

        let config = indoc! {r#"
            # The extension config is added automatically by the test runner.
            # Add here any additional configuration for the Grafbase Gateway.
        "#};

        // The test configuration is built with the subgraph and networking enabled.
        // You must have the CLI and Grafbase Gateway for this to work. If you do not have
        // them in the PATH, you can specify the paths to the executables with the `.with_cli` and
        // `.with_gateway` methods.
        let config = TestConfig::builder()
            .with_subgraph(subgraph)
            .enable_networking()
            .build(config)
            .unwrap();

        // A runner for building the extension, and executing the Grafbase Gateway together
        // with the subgraphs. The runner composes all subgraphs into a federated schema.
        let runner = TestRunner::new(config).await.unwrap();

        let result: serde_json::Value = runner
            .graphql_query(r#"query { hi }"#)
            .send()
            .await
            .unwrap();

        // The result is compared against a snapshot.
        insta::assert_json_snapshot!(result, @r#"
        {
          "data": {
            "hi": "hello"
          }
        }
        "#);
    }
    "##);
}

#[test]
fn build() {
    // FIXME: Make this test work on windows, linux arm64 and macos x86
    if cfg!(windows)
        || (cfg!(target_arch = "aarch64") && cfg!(target_os = "linux"))
        || (cfg!(target_arch = "x86_64") && cfg!(target_os = "macos"))
    {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", "--type", "authorization", &*project_path_str];
    let command = cmd(cargo_bin("grafbase"), &args).stdout_null().stderr_null();
    command.run().unwrap();

    use_latest_grafbase_sdk_in_cargo_toml(&project_path);

    let result = cmd("cargo", &["check", "--tests"])
        .env("RUSTFLAGS", "")
        .dir(&project_path)
        .unchecked()
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();

    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );

    let args = vec!["extension", "build"];

    let result = cmd(cargo_bin("grafbase"), &args)
        // we do -D warnings in CI, the template has unused variable warnings...
        .env("RUSTFLAGS", "")
        .dir(&project_path)
        .unchecked()
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();

    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );

    let build_path = project_path.join("build");
    assert!(std::fs::exists(build_path.join("extension.wasm")).unwrap());
    assert!(std::fs::exists(build_path.join("manifest.json")).unwrap());

    let manifest = std::fs::read_to_string(build_path.join("manifest.json")).unwrap();
    let manifest: VersionedManifest = serde_json::from_str(&manifest).unwrap();

    let manifest = serde_json::to_value(&manifest).unwrap();
    insta::assert_json_snapshot!(
        manifest,
        {
            ".sdk_version" => "<sdk_version>",
            ".minimum_gateway_version" => "<minimum_gateway_version>"
        },
        @r#"
    {
      "manifest": "v1",
      "id": {
        "name": "test-project",
        "version": "0.1.0"
      },
      "kind": {
        "Authorization": {}
      },
      "sdk_version": "<sdk_version>",
      "minimum_gateway_version": "<minimum_gateway_version>",
      "description": "A new extension",
      "sdl": "\"\"\"\nFill in here the directives and types that the extension needs.\n\"\"\"\ndirective @testProjectConfiguration(arg1: String) repeatable on SCHEMA\ndirective @testProjectDirective on FIELD_DEFINITION",
      "permissions": []
    }
    "#
    );
}
