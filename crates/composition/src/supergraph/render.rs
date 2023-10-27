use crate::{strings::Strings, subgraphs::DefinitionKind, StringId, Supergraph};
use std::fmt::Write as _;

/// This cannot fail, other than on a format error, and does not produce diagnostics.
impl Supergraph {
    pub(crate) fn render(&self, strings: &Strings) -> String {
        let mut out = String::new();
        for (definition_name, definition_kind) in &self.definitions {
            match definition_kind {
                DefinitionKind::Object => {
                    writeln!(out, "type {} {{", &strings[*definition_name]).unwrap();

                    for ((_, field_name), field_type) in self
                        .fields
                        .range((*definition_name, StringId::MIN)..(*definition_name, StringId::MAX))
                    {
                        writeln!(
                            out,
                            "    {}: {}",
                            &strings[*field_name], &strings[*field_type]
                        )
                        .unwrap();
                    }

                    out.push_str("}\n");
                }
                _ => todo!(),
            }
        }
        out
    }
}
