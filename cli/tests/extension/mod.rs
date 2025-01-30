use duct::cmd;
use extension::Manifest;
use tempfile::tempdir;

use crate::cargo_bin;

#[test]
fn init() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", &*project_path_str];

    let command = cmd(cargo_bin("grafbase"), &args).stdout_null().stderr_null();

    command.run().unwrap();

    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();

    insta::assert_snapshot!(&cargo_toml, @r#"
    [package]
    name = "test-project"
    version = "0.1.0"
    edition = "2021"
    license = "Apache-2.0"

    [dependencies]
    grafbase-sdk = "0.1.2"

    [lib]
    crate-type = ["cdylib"]
    "#);

    let definitions = std::fs::read_to_string(project_path.join("definitions.graphql")).unwrap();

    insta::assert_snapshot!(&definitions, @r#"
    """
    Fill in here the directives and types that the extension needs.
    Remove this file and the definition in extension.toml if the extension does not need any directives.
    """
    directive @testProjectConfiguration(arg1: String) repeatable on SCHEMA
    directive @testProjectDirective on FIELD_DEFINITION
    "#);

    let extension_toml = std::fs::read_to_string(project_path.join("extension.toml")).unwrap();

    insta::assert_snapshot!(&extension_toml, @r#"
    [extension]
    name = "test-project"
    version = "0.1.0"
    kind = "resolver"

    [directives]
    definitions = "definitions.graphql"
    field_resolvers = ["testProjectDirective"]
    "#);

    let lib_rs = std::fs::read_to_string(project_path.join("src/lib.rs")).unwrap();

    insta::assert_snapshot!(&lib_rs, @r##"
    use grafbase_sdk::{
        types::{Directive, FieldDefinition, FieldInputs, FieldOutput},
        Error, Extension, Resolver, ResolverExtension, SharedContext,
    };

    #[derive(ResolverExtension)]
    struct TestProject;

    impl Extension for TestProject {
        fn new(schema_directives: Vec<Directive>) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self)
        }
    }

    impl Resolver for TestProject {
        fn resolve_field(
            &mut self,
            context: SharedContext,
            directive: Directive,
            field_definition: FieldDefinition,
            inputs: FieldInputs,
        ) -> Result<FieldOutput, Error> {
            todo!()
        }
    }
    "##);
}

#[test]
fn build() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", &*project_path_str];
    let command = cmd(cargo_bin("grafbase"), &args).stdout_null().stderr_null();
    command.run().unwrap();

    let args = vec!["extension", "build"];

    let command = cmd(cargo_bin("grafbase"), &args)
        // we do -D warnings in CI, the template has unused variable warnings...
        .env("RUSTFLAGS", "")
        .dir(&project_path)
        .stderr_null()
        .stdout_null();

    command.run().unwrap();

    let build_path = project_path.join("build");
    assert!(std::fs::exists(build_path.join("extension.wasm")).unwrap());
    assert!(std::fs::exists(build_path.join("manifest.json")).unwrap());

    let manifest = std::fs::read_to_string(build_path.join("manifest.json")).unwrap();
    let manifest: Manifest = serde_json::from_str(&manifest).unwrap();

    let manifest = serde_json::to_value(&manifest).unwrap();
    insta::assert_json_snapshot!(
        manifest,
        {
            ".sdk_version" => "<sdk_version>",
            ".minimum_gateway_version" => "<minimum_gateway_version>"
        },
        @r#"
    {
      "name": "test-project",
      "version": "0.1.0",
      "kind": {
        "FieldResolver": {
          "resolver_directives": [
            "testProjectDirective"
          ]
        }
      },
      "sdk_version": "<sdk_version>",
      "minimum_gateway_version": "<minimum_gateway_version>",
      "sdl": "\"\"\"\nFill in here the directives and types that the extension needs.\nRemove this file and the definition in extension.toml if the extension does not need any directives.\n\"\"\"\ndirective @testProjectConfiguration(arg1: String) repeatable on SCHEMA\ndirective @testProjectDirective on FIELD_DEFINITION"
    }
    "#
    );
}
