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

                    for ((_, field_name), field) in self
                        .fields
                        .range((*definition_name, StringId::MIN)..(*definition_name, StringId::MAX))
                    {
                        let args = render_field_arguments(&field.arguments, strings);
                        writeln!(
                            out,
                            "    {}{args}: {}",
                            &strings[*field_name], &strings[field.field_type]
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
                    for ((_, field_name), field) in fields {
                        writeln!(
                            out,
                            "    {}: {}",
                            &strings[*field_name], &strings[field.field_type]
                        )
                        .unwrap();
                    }

                    out.push_str("}\n");
                }

                DefinitionKind::Interface => {
                    writeln!(out, "interface {} {{", &strings[*definition_name]).unwrap();

                    for ((_, field_name), field) in self
                        .fields
                        .range((*definition_name, StringId::MIN)..(*definition_name, StringId::MAX))
                    {
                        let args = render_field_arguments(&field.arguments, strings);
                        writeln!(
                            out,
                            "    {}{args}: {}",
                            &strings[*field_name], &strings[field.field_type]
                        )
                        .unwrap();
                    }
                    out.push_str("}\n");
                }

                DefinitionKind::Scalar => {
                    writeln!(out, "scalar {}", strings.resolve(*definition_name)).unwrap();
                }

                DefinitionKind::Enum => {
                    writeln!(out, "enum {} {{", strings.resolve(*definition_name)).unwrap();

                    for (_, value) in self
                        .enum_values
                        .range((*definition_name, StringId::MIN)..(*definition_name, StringId::MAX))
                    {
                        writeln!(out, "  {}", strings.resolve(*value)).unwrap();
                    }

                    out.push_str("}\n");
                }
            }
        }
        out
    }
}

fn render_field_arguments(args: &[(StringId, StringId)], strings: &Strings) -> String {
    if args.is_empty() {
        String::new()
    } else {
        let inner = args
            .iter()
            .map(|(name, ty)| (strings.resolve(*name), strings.resolve(*ty)))
            .map(|(name, ty)| format!("{name}: {ty}"))
            .join(", ");
        format!("({inner})")
    }
}
