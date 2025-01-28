use duct::cmd;
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
    grafbase-sdk = "0.1.0"

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
    name = "testProject"
    version = "0.1.0"

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
