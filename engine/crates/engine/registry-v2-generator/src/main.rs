mod exts;
mod file;
mod idents;
mod object;
mod union;

use anyhow::Context;
use indexmap::IndexMap;
use indoc::formatdoc;
use itertools::Itertools;

use cynic_parser::type_system::{Definition, TypeDefinition};

use crate::{exts::FileDirectiveExt, file::imports};

const ENGINE_DIR: &str = "engine/crates/engine";

fn main() -> anyhow::Result<()> {
    eprintln!("{:?}", std::env::current_dir());
    for module in ["registry"] {
        let module_path = format!("{ENGINE_DIR}/registry-v2/src/generated");
        std::fs::create_dir_all(&module_path).with_context(|| format!("creating {module_path}"))?;

        let document = std::fs::read_to_string(format!("{ENGINE_DIR}/registry-v2-generator/domain/{module}.graphql"))?;

        let domain = match cynic_parser::parse_type_system_document(&document) {
            Ok(domain) => domain,
            Err(error) => {
                eprintln!("Error parsing document");
                eprintln!("{}", error.to_report(&document));
                return Err(anyhow::anyhow!(""));
            }
        };

        let mut model_index = IndexMap::new();

        for definition in domain.definitions() {
            match definition {
                Definition::Type(ty) => {
                    model_index.insert(ty.name(), ty);
                }
                _ => anyhow::bail!("unsupported definition"),
            }
        }

        let id_trait = match module {
            "registry" => "RegistryId",
            _ => unimplemented!("id_trait for {module} needs defined (see the location of this panic)"),
        };

        let outputs = model_index
            .values()
            .map(|model| {
                let output = match model {
                    TypeDefinition::Object(object) => object::object_output(*object, &model_index, id_trait)?,
                    TypeDefinition::Scalar(_) => {
                        return Ok(None);
                    }
                    TypeDefinition::Union(union) => union::union_output(*union, &model_index, id_trait)?,
                    _ => anyhow::bail!("unsupported definition"),
                };

                Ok(Some((model.file_name(), output)))
            })
            .filter_map(Result::transpose)
            .collect::<Result<Vec<(_, _)>, _>>()
            .unwrap()
            .into_iter()
            .into_group_map();

        for (file_name, output) in outputs {
            let requires = output
                .iter()
                .flat_map(|entity| entity.requires.clone().into_iter())
                .collect();
            let current_entities = output.iter().map(|entity| entity.id.clone()).collect();

            let imports = imports(requires, current_entities, shared_imports()).unwrap();

            let doc = format_code(formatdoc!(
                r#"
                {imports}

                {}
                "#,
                output.into_iter().map(|entity| entity.contents).join("\n\n")
            ))
            .unwrap();

            std::fs::write(format!("{module_path}/{file_name}.rs"), doc).unwrap();
        }
    }

    Ok(())
}

fn format_code(text: impl ToString) -> anyhow::Result<String> {
    use xshell::{cmd, Shell};
    let sh = Shell::new()?;

    let stdout = cmd!(sh, "rustfmt").stdin(&text.to_string()).read()?;

    Ok(stdout)
}

fn shared_imports() -> proc_macro2::TokenStream {
    quote::quote! {
        use super::prelude::*;
    }
}
