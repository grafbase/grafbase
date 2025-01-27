use std::path::Path;

use askama::Template;
use convert_case::{Case, Casing};

use crate::cli_input::{ExtensionCommand, ExtensionInitCommand, ExtensionSubCommand};

pub(crate) fn extension(cmd: ExtensionCommand) -> anyhow::Result<()> {
    match cmd.command {
        ExtensionSubCommand::Init(cmd) => init(cmd),
    }
}

fn init(cmd: ExtensionInitCommand) -> anyhow::Result<()> {
    if cmd.path.exists() {
        anyhow::bail!("destination '{}' already exists", cmd.path.to_string_lossy());
    }

    std::fs::create_dir_all(&cmd.path)?;

    let extension_name = init_cargo_toml(&cmd.path)?;
    init_extension_toml(&cmd.path, &extension_name)?;
    init_definitions_graphql(&cmd.path, &extension_name)?;
    init_lib_rs(&cmd.path, &extension_name)?;
    init_gitignore(&cmd.path)?;

    Ok(())
}

fn init_gitignore(path: &Path) -> anyhow::Result<()> {
    let gitignore_path = path.join(".gitignore");
    let contents = indoc::indoc! {r#"
        target
        build
    "#};

    std::fs::write(gitignore_path, contents)?;

    Ok(())
}

fn init_lib_rs(path: &Path, extension_name: &str) -> anyhow::Result<()> {
    #[derive(askama::Template)]
    #[template(path = "extension/src/lib.rs.template", escape = "none")]
    struct LibRsTemplate<'a> {
        name: &'a str,
    }

    let struct_name = extension_name.to_case(Case::Pascal);
    let lib_rs_path = path.join("src");

    std::fs::create_dir(&lib_rs_path)?;

    let mut writer = std::fs::File::create(lib_rs_path.join("lib.rs"))?;
    LibRsTemplate { name: &struct_name }.write_into(&mut writer)?;

    Ok(())
}

fn init_definitions_graphql(path: &Path, extension_name: &str) -> anyhow::Result<()> {
    #[derive(askama::Template)]
    #[template(path = "extension/definitions.graphql.template", escape = "none")]
    struct GraphQLDefinitionsTemplate<'a> {
        name: &'a str,
    }

    let name = extension_name.to_case(Case::Camel);

    let mut writer = std::fs::File::create(path.join("definitions.graphql"))?;
    GraphQLDefinitionsTemplate { name: &name }.write_into(&mut writer)?;

    Ok(())
}

fn init_cargo_toml(project_path: &Path) -> anyhow::Result<String> {
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

    let cargo_toml_path = project_path.join("Cargo.toml");

    let sdk_cargo_toml = include_str!("../../crates/grafbase-sdk/Cargo.toml");
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

fn init_extension_toml(project_path: &Path, extension_name: &str) -> anyhow::Result<()> {
    #[derive(askama::Template)]
    #[template(path = "extension/extension.toml.template", escape = "none")]
    struct ExtensionTomlTemplate<'a> {
        name: &'a str,
    }

    let extension_toml_path = project_path.join("extension.toml");
    let camel_case_extension = extension_name.to_case(Case::Camel);

    let mut writer = std::fs::File::create(&extension_toml_path)?;

    let template = ExtensionTomlTemplate {
        name: &camel_case_extension,
    };

    template.write_into(&mut writer)?;

    Ok(())
}
