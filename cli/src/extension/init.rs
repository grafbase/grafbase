use std::path::Path;

use askama::Template;
use convert_case::{Case, Casing};

use crate::cli_input::{ExtensionInitCommand, ExtensionType};

#[derive(askama::Template)]
#[template(path = "extension/src/resolver.rs.template", escape = "none")]
struct ResolverTemplate<'a> {
    name: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/src/authentication.rs.template", escape = "none")]
struct AuthenticationTemplate<'a> {
    name: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/src/authorization.rs.template", escape = "none")]
struct AuthorizationTemplate<'a> {
    name: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/src/hooks.rs.template", escape = "none")]
struct HooksTemplate<'a> {
    name: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/src/contracts.rs.template", escape = "none")]
struct ContractsTemplate<'a> {
    name: &'a str,
}

#[derive(serde::Deserialize)]
struct SdkCargoToml {
    package: SdkCargoTomlPackage,
}

#[derive(serde::Deserialize)]
struct SdkCargoTomlPackage {
    version: String,
}

#[derive(askama::Template)]
#[template(path = "extension/Cargo.toml.template", escape = "none")]
struct CargoTomlTemplate<'a> {
    name: &'a str,
    sdk_version: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/definitions.graphql.template", escape = "none")]
struct GraphQLDefinitionsTemplate<'a> {
    name: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/extension.toml.template", escape = "none")]
struct ExtensionTomlTemplate<'a> {
    name: &'a str,
    r#type: ExtensionType,
    type_config: &'a str,
}

#[derive(askama::Template)]
#[template(path = "extension/tests/integration_tests.rs.template", escape = "none")]
struct IntegrationTestsTemplate;

pub(super) fn execute(cmd: ExtensionInitCommand) -> anyhow::Result<()> {
    if cmd.path.exists() {
        anyhow::bail!("destination '{}' already exists", cmd.path.to_string_lossy());
    }

    std::fs::create_dir_all(&cmd.path)?;

    let extension_name = init_cargo_toml(&cmd.path)?;
    init_extension_toml(&cmd.path, cmd.r#type, &extension_name)?;

    if matches!(
        cmd.r#type,
        ExtensionType::Resolver | ExtensionType::Authorization | ExtensionType::Contracts
    ) {
        init_definitions_graphql(&cmd.path, &extension_name)?;
    }

    init_rust_files(&cmd.path, cmd.r#type, &extension_name)?;
    init_gitignore(&cmd.path)?;

    Ok(())
}

fn init_gitignore(path: &Path) -> anyhow::Result<()> {
    let gitignore_path = path.join(".gitignore");

    let contents = indoc::indoc! {r#"
        target
        build
        .build.lock
    "#};

    std::fs::write(gitignore_path, contents)?;

    Ok(())
}

fn init_rust_files(path: &Path, extension_type: ExtensionType, extension_name: &str) -> anyhow::Result<()> {
    let struct_name = extension_name.to_case(Case::Pascal);
    let lib_rs_path = path.join("src");

    std::fs::create_dir(&lib_rs_path)?;

    let mut writer = std::fs::File::create(lib_rs_path.join("lib.rs"))?;

    match extension_type {
        ExtensionType::Resolver => ResolverTemplate { name: &struct_name }.write_into(&mut writer)?,
        ExtensionType::Authentication => AuthenticationTemplate { name: &struct_name }.write_into(&mut writer)?,
        ExtensionType::Authorization => AuthorizationTemplate { name: &struct_name }.write_into(&mut writer)?,
        ExtensionType::Hooks => HooksTemplate { name: &struct_name }.write_into(&mut writer)?,
        ExtensionType::Contracts => ContractsTemplate { name: &struct_name }.write_into(&mut writer)?,
    }

    let tests_path = path.join("tests");
    std::fs::create_dir(&tests_path)?;

    let mut writer = std::fs::File::create(tests_path.join("integration_tests.rs"))?;
    IntegrationTestsTemplate.write_into(&mut writer)?;

    Ok(())
}

fn init_definitions_graphql(path: &Path, extension_name: &str) -> anyhow::Result<()> {
    let name = extension_name.to_case(Case::Camel);

    let mut writer = std::fs::File::create(path.join("definitions.graphql"))?;
    GraphQLDefinitionsTemplate { name: &name }.write_into(&mut writer)?;

    Ok(())
}

fn init_cargo_toml(project_path: &Path) -> anyhow::Result<String> {
    let cargo_toml_path = project_path.join("Cargo.toml");

    let sdk_cargo_toml = include_str!("../../../crates/grafbase-sdk/Cargo.toml");
    let sdk_cargo_toml: SdkCargoToml = toml::from_str(sdk_cargo_toml)?;

    let name = project_path
        .file_name()
        .expect("must_exist")
        .to_string_lossy()
        .to_case(Case::Kebab);

    let template = CargoTomlTemplate {
        name: &name,
        sdk_version: &sdk_cargo_toml.package.version,
    };

    let mut writer = std::fs::File::create(&cargo_toml_path)?;
    template.write_into(&mut writer)?;

    Ok(name)
}

fn init_extension_toml(project_path: &Path, r#type: ExtensionType, extension_name: &str) -> anyhow::Result<()> {
    let extension_toml_path = project_path.join("extension.toml");

    let mut writer = std::fs::File::create(&extension_toml_path)?;

    let template = ExtensionTomlTemplate {
        name: extension_name,
        r#type,
        type_config: match r#type {
            ExtensionType::Resolver => indoc::indoc! { r#"
                [resolver]
                # Directives that resolve to data. Every other directive will be treated as metadata only.
                # Defaults to all defined directives.
                #
                # directives = ["resolve"]
            "#},
            ExtensionType::Authentication => "",
            ExtensionType::Authorization => indoc::indoc! { r#"
                [authorization]
                # Directives that need to granted authorization during execution. Every other directive will be treated as metadata only.
                # Defaults to all defined directives.
                #
                # directives = ["myAuth"]

                # Refine the grouping used by the gateway for authorization. By default, the gateway will request authorization only once for
                # any decorated element (ex: field), independent of its location in the query, from which subgraph it comes from 
                # and how many times it appears in the query.
                #
                # Supported values include:
                # - `subgraph`: The gateway will request authorization for each subgraph that contains the decorated element.
                #
                # Defaults to empty list.
                #
                # group_by = ["subgraph"]
            "# },
            ExtensionType::Hooks => indoc::indoc! { r#"
                [hooks]
                # Specify the events you intend to read in the on_response hook.
                # Defaults to no events.
                #
                # events = "*"
                # events = ["operation", "http_request"]
            "#},
            ExtensionType::Contracts => "",
        },
    };

    template.write_into(&mut writer)?;

    Ok(())
}
