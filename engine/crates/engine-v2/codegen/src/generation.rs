mod docstr;
mod id;
mod imports;
mod module;
mod object;
mod union;

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use anyhow::Ok;
use imports::generate_imports;
pub use module::generate_module_base_content;
use object::generate_object;
use proc_macro2::TokenStream;
use tracing::info_span;
use union::generate_union;

use crate::domain::{Definition, Domain};
use crate::formatter::Formatter;

struct GeneratedCode<'a> {
    module_path: &'a [String],
    imports: Imports<'a>,
    code_sections: Vec<TokenStream>,
}

#[derive(Default)]
struct Imports<'a> {
    local: HashSet<&'a str>,
    generated: HashSet<&'a str>,
    walker_lib: HashSet<&'a str>,
}

impl<'a> Imports<'a> {
    fn extend(&mut self, other: Self) {
        self.generated.extend(other.generated);
        self.walker_lib.extend(other.walker_lib);
    }
}

#[derive(Default)]
struct Module<'a> {
    submodules: Vec<&'a str>,
    imports: Imports<'a>,
    code_sections: Vec<TokenStream>,
}

pub struct GeneratedModule {
    pub module_path: Vec<String>,
    pub contents: String,
}

pub fn generate_modules(formatter: &Formatter, domain: &Domain) -> anyhow::Result<Vec<GeneratedModule>> {
    let mut modules = BTreeMap::<_, Module<'_>>::new();
    let mut names = domain
        .definitions_by_name
        .iter()
        .filter_map(|(name, definition)| {
            if definition.external_domain_name().is_some() {
                return None;
            }
            match definition {
                Definition::Scalar(_) => None,
                Definition::Object(def) => Some((def.span.start, name)),
                Definition::Union(def) => Some((def.span.start, name)),
            }
        })
        .collect::<Vec<_>>();

    // Ensure consistent ordering of generated code despite the hashmap
    names.sort_unstable_by_key(|(start, _)| *start);

    for (_, name) in names {
        let definition = &domain.definitions_by_name[name];
        let generated_code = match definition {
            Definition::Scalar(_) => unreachable!(),
            Definition::Object(object) => generate_object(domain, object)?,
            Definition::Union(union) => generate_union(domain, union)?,
        };
        let GeneratedCode {
            module_path,
            imports,
            code_sections,
        } = generated_code;

        let module = modules.entry(module_path).or_default();
        module.imports.extend(imports);
        module.imports.local.insert(name);
        module.code_sections.extend(code_sections);

        if module_path.len() > 1 {
            let parent = &module_path[..module_path.len() - 1];
            modules
                .entry(parent)
                .or_default()
                .submodules
                .push(module_path.last().unwrap());
        } else if module_path.is_empty() {
            tracing::warn!("No module defined for '{name}'? Do we even support it?");
        }
    }

    modules
        .into_iter()
        .map(
            |(
                module_path,
                Module {
                    submodules,
                    imports,
                    code_sections,
                },
            )| {
                let _guard = info_span!("module_generation", ?module_path).entered();
                let mut contents = generate_module_base_content(domain, &submodules);

                if !code_sections.is_empty() {
                    write!(contents, "{}", generate_imports(domain, module_path, imports)?)?;

                    for code_section in code_sections {
                        write!(contents, "\n\n{}", code_section)?;
                    }
                }

                let contents = formatter.format(contents)?;

                Ok(GeneratedModule {
                    module_path: module_path.to_vec(),
                    contents,
                })
            },
        )
        .collect::<Result<Vec<_>, _>>()
}
