mod authorization;
mod install;
mod publish;
mod update;

use duct::cmd;
use extension::VersionedManifest;
use std::path::Path;
use tempfile::tempdir;

use crate::cargo_bin;

#[test]
fn init_resolver() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", "--type", "resolver", &*project_path_str];

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
    grafbase-sdk = "0.17.5"
    serde = { version = "1", features = ["derive"] }

    [dev-dependencies]
    insta = { version = "1", features = ["json"] }
    grafbase-sdk = { version = "0.17.5", features = ["test-utils"] }
    tokio = { version = "1", features = ["rt-multi-thread", "macros", "test-util"] }
    serde_json = "1"
    "#);

    let definitions = std::fs::read_to_string(project_path.join("definitions.graphql")).unwrap();

    insta::assert_snapshot!(&definitions, @r#"
    """
    Fill in here the directives and types that the extension needs.
    """
    directive @testProjectConfiguration(arg1: String) repeatable on SCHEMA
    directive @testProjectDirective on FIELD_DEFINITION
    "#);

    let extension_toml = std::fs::read_to_string(project_path.join("extension.toml")).unwrap();

    insta::assert_snapshot!(&extension_toml, @r##"
    [extension]
    name = "test-project"
    version = "0.1.0"
    type = "resolver"
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
        ResolverExtension,
        types::{Configuration, Error, ResolvedField, Response, SubgraphHeaders, SubgraphSchema, Variables},
    };

    #[derive(ResolverExtension)]
    struct TestProject {
        config: Config
    }

    // Configuration in the TOML for this extension
    #[derive(serde::Deserialize)]
    struct Config {
        #[serde(default)]
        key: Option<String>
    }

    impl ResolverExtension for TestProject {
        fn new(subgraph_schemas: Vec<SubgraphSchema>, config: Configuration) -> Result<Self, Error> {
            let config: Config = config.deserialize()?;
            Ok(Self { config })
        }

        fn resolve(
            &mut self,
            prepared: &[u8],
            headers: SubgraphHeaders,
            variables: Variables,
        ) -> Result<Response, Error> {
            // field which must be resolved. The prepared bytes can be customized to store anything you need in the operation cache.
            let field = ResolvedField::try_from(prepared)?;
            Ok(Response::null())
        }
    }
    "##);

    let tests_rs = std::fs::read_to_string(project_path.join("tests/integration_tests.rs")).unwrap();

    insta::assert_snapshot!(&tests_rs, @r##"
    use grafbase_sdk::test::{GraphqlSubgraph, TestGateway};

    #[tokio::test]
    async fn test_example() {
        // You must have the CLI and Grafbase Gateway for this to work. If you do not have
        // them in the PATH, you can specify the paths to the executables with the `.with_cli` and
        // `.with_gateway` methods.
        let gateway = TestGateway::builder()
            .subgraph(
                GraphqlSubgraph::with_schema(
                    r#"
                    type Query {
                        hi: String
                    }
                    "#,
                )
                .with_resolver("Query", "hi", "Alice"),
            )
            .toml_config(
                r#"
                # The extension config is added automatically by the test runner.
                # Add here any additional configuration for the Grafbase Gateway.
                "#,
            )
            .build()
            .await
            .unwrap();

        let response = gateway.query(r#"query { hi }"#).send().await;

        // The result is compared against a snapshot.
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "hi": "Alice"
          }
        }
        "#);
    }
    "##);
}

#[test]
fn build_resolver() {
    // FIXME: Make this test work on windows, linux arm64 and darwin x86.
    if cfg!(windows)
        || (cfg!(target_arch = "aarch64") && cfg!(target_os = "linux"))
        || (cfg!(target_arch = "x86_64") && cfg!(target_os = "macos"))
    {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", "--type", "resolver", &*project_path_str];
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
      "type": {
        "Resolver": {}
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

#[test]
fn init_auth() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", "--type", "authentication", &*project_path_str];

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
    grafbase-sdk = "0.17.5"
    serde = { version = "1", features = ["derive"] }

    [dev-dependencies]
    insta = { version = "1", features = ["json"] }
    grafbase-sdk = { version = "0.17.5", features = ["test-utils"] }
    tokio = { version = "1", features = ["rt-multi-thread", "macros", "test-util"] }
    serde_json = "1"
    "#);

    let extension_toml = std::fs::read_to_string(project_path.join("extension.toml")).unwrap();

    insta::assert_snapshot!(&extension_toml, @r##"
    [extension]
    name = "test-project"
    version = "0.1.0"
    type = "authentication"
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
        AuthenticationExtension,
        types::{Configuration, Error, ErrorResponse, GatewayHeaders, Token},
    };

    #[derive(AuthenticationExtension)]
    struct TestProject;

    impl AuthenticationExtension for TestProject {
        fn new(config: Configuration) -> Result<Self, Error> {
            Ok(Self)
        }

        fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
            Err(ErrorResponse::unauthorized())
        }
    }
    "##);

    let tests_rs = std::fs::read_to_string(project_path.join("tests/integration_tests.rs")).unwrap();

    insta::assert_snapshot!(&tests_rs, @r##"
    use grafbase_sdk::test::{GraphqlSubgraph, TestGateway};

    #[tokio::test]
    async fn test_example() {
        // You must have the CLI and Grafbase Gateway for this to work. If you do not have
        // them in the PATH, you can specify the paths to the executables with the `.with_cli` and
        // `.with_gateway` methods.
        let gateway = TestGateway::builder()
            .subgraph(
                GraphqlSubgraph::with_schema(
                    r#"
                    type Query {
                        hi: String
                    }
                    "#,
                )
                .with_resolver("Query", "hi", "Alice"),
            )
            .toml_config(
                r#"
                # The extension config is added automatically by the test runner.
                # Add here any additional configuration for the Grafbase Gateway.
                "#,
            )
            .build()
            .await
            .unwrap();

        let response = gateway.query(r#"query { hi }"#).send().await;

        // The result is compared against a snapshot.
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "hi": "Alice"
          }
        }
        "#);
    }
    "##);
}

#[test]
fn build_auth() {
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

    let args = vec!["extension", "init", "--type", "authentication", &*project_path_str];
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
      "type": {
        "Authentication": {}
      },
      "sdk_version": "<sdk_version>",
      "minimum_gateway_version": "<minimum_gateway_version>",
      "description": "A new extension",
      "permissions": []
    }
    "#
    );
}

fn use_latest_grafbase_sdk_in_cargo_toml(project_path: &Path) {
    let grafbase_sdk_dir = format!("{}/../crates/grafbase-sdk", env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();

    let regex = regex::Regex::new(r#"grafbase-sdk\s*=\s*".*""#).unwrap();
    let cargo_toml = regex.replace_all(
        &cargo_toml,
        format!(r#"grafbase-sdk = {{ path = "{grafbase_sdk_dir}" }}"#),
    );

    let regex = regex::Regex::new(r#"grafbase-sdk\s*=\s*\{\s*version\s*=\s*".*?"(.*)\}"#).unwrap();
    let cargo_toml = regex.replace_all(
        &cargo_toml,
        format!(r#"grafbase-sdk = {{ path = "{grafbase_sdk_dir}", features = ["test-utils"] }}"#),
    );
    println!("{cargo_toml}");

    std::fs::write(project_path.join("Cargo.toml"), cargo_toml.as_ref()).unwrap();
}
