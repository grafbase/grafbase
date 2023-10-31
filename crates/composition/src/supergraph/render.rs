use crate::{strings::Strings, subgraphs::DefinitionKind, StringId, Supergraph};
use itertools::Itertools;
use std::fmt::Write as _;

/// This cannot fail, other than on a format error, and does not produce diagnostics.
impl Supergraph {
    pub(crate) fn render(&self, strings: &Strings) -> String {
        let mut out = String::new();
        for (definition_name, definition_kind) in &self.definitions {
            match definition_kind {
                DefinitionKind::Object => {
                    writeln!(out, "type {} {{", &strings[*definition_name]).unwrap();

                    for ((_, field_name), (args, field_type)) in self
                        .fields
                        .range((*definition_name, StringId::MIN)..(*definition_name, StringId::MAX))
                    {
                        let args = if args.is_empty() {
                            String::new()
                        } else {
                            let inner = args
                                .iter()
                                .map(|(name, ty)| (strings.resolve(*name), strings.resolve(*ty)))
                                .map(|(name, ty)| format!("{name}: {ty}"))
                                .join(", ");
                            format!("({inner})")
                        };
                        writeln!(
                            out,
                            "    {}{args}: {}",
                            &strings[*field_name], &strings[*field_type]
                        )
                        .unwrap();
                    }

                    out.push_str("}\n");
                }

                DefinitionKind::Union => {
                    write!(out, "union {} = ", &strings[*definition_name]).unwrap();
                    let members = self
                        .union_members
                        .range((*definition_name, StringId::MIN)..(*definition_name, StringId::MAX))
                        .map(|(_, member)| &strings[*member])
                        .join(" | ");
                    writeln!(out, "{members}").unwrap();
                }

                DefinitionKind::InputObject => {
                    writeln!(out, "input {} {{", &strings[*definition_name]).unwrap();

                    let fields = self.fields.range(
                        (*definition_name, StringId::MIN)..(*definition_name, StringId::MAX),
                    );
                    for ((_, field_name), (_, field_type)) in fields {
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
