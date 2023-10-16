use crate::analyze::{AnalyzedSchema, BuiltinScalar, Definition, Field, FieldTypeKind, ListWrapper, ObjectKind};
use std::fmt;

const INDENT: &str = "  ";

pub(crate) fn generate_module<O>(schema: &AnalyzedSchema<'_>, out: &mut O) -> fmt::Result
where
    O: fmt::Write,
{
    for (idx, definition) in schema.definitions.iter().enumerate() {
        match definition {
            Definition::Object(id) => {
                let object_type = &schema[*id];
                let graphql_type_name = object_type.name;
                let ts_type_name = safe_ts_type_name(graphql_type_name);
                let is_input_object = matches!(object_type.kind, ObjectKind::InputObject);

                maybe_docs(out, object_type.docs, "")?;
                writeln!(out, "export type {ts_type_name} = {{")?;

                if let ObjectKind::Object = object_type.kind {
                    writeln!(out, "{INDENT}__typename?: '{graphql_type_name}';")?;
                }

                for field in schema.iter_object_fields(*id) {
                    let field_optional = !is_input_object
                        && matches!(
                            field.kind,
                            FieldTypeKind::Definition(Definition::Object(_) | Definition::Union(_))
                        );

                    let field_optional = if field_optional { "?" } else { "" };
                    let field_name = field.name;
                    let field_type = render_field_type(field, schema);

                    maybe_docs(out, field.docs, INDENT)?;
                    writeln!(out, "{INDENT}{field_name}{field_optional}: {field_type};")?;
                }

                out.write_str("};\n")?;
            }
            Definition::Union(union_id) => {
                let union_name = schema[*union_id].name;
                write!(out, "export type {union_name} = ")?;

                let mut variants = schema.iter_union_variants(*union_id).peekable();
                while let Some(variant) = variants.next() {
                    write!(out, "{}", safe_ts_type_name(variant.name))?;

                    if variants.peek().is_some() {
                        out.write_str(" | ")?;
                    }
                }
                out.write_str(";\n")?;
            }
            Definition::Enum(enum_id) => {
                let r#enum = &schema[*enum_id];
                let enum_name = safe_ts_type_name(r#enum.name);
                maybe_docs(out, r#enum.docs, "")?;
                writeln!(out, "export enum {enum_name} {{")?;
                for variant in schema.iter_enum_variants(*enum_id) {
                    out.write_str(INDENT)?;
                    out.write_str(variant)?;
                    out.write_str(",\n")?;
                }
                out.write_str("}\n")?;
            }
            Definition::CustomScalar(id) => {
                let scalar = &schema[*id];
                let scalar_name = safe_ts_type_name(scalar.name);
                maybe_docs(out, scalar.docs, "")?;
                writeln!(out, "export type {scalar_name} = any;")?;
            }
        }

        // Newline between definitions but not at the end.
        if idx + 1 < schema.definitions.len() {
            writeln!(out)?;
        }
    }

    Ok(())
}

fn render_field_type(field: &Field<'_>, schema: &AnalyzedSchema<'_>) -> String {
    let mut type_string = match &field.kind {
        FieldTypeKind::BuiltinScalar(scalar) => match scalar {
            BuiltinScalar::Int | BuiltinScalar::Float => "number",
            BuiltinScalar::String | BuiltinScalar::Id => "string",
            BuiltinScalar::Boolean => "boolean",
        }
        .to_owned(),
        FieldTypeKind::Definition(def) => safe_ts_type_name(match *def {
            Definition::CustomScalar(id) => schema[id].name,
            Definition::Enum(id) => schema[id].name,
            Definition::Object(id) => schema[id].name,
            Definition::Union(id) => schema[id].name,
        })
        .to_string(),
    };

    if field.inner_is_nullable() {
        type_string.push_str(" | null");
    }

    for wrapper in field.iter_list_wrappers() {
        type_string = match wrapper {
            ListWrapper::NullableList => format!("Array<{type_string}> | null"),
            ListWrapper::NonNullList => format!("Array<{type_string}>"),
        }
    }

    type_string
}

/// Takes a GraphQL name, and returns a typescript-safe version. This takes TypeScript reserved keywords into account.
fn safe_ts_type_name(graphql_name: &str) -> impl fmt::Display + '_ {
    struct SafeName<'a>(&'a str);

    impl fmt::Display for SafeName<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.0 {
                "any" | "async" | "boolean" | "interface" | "never" | "null" | "number" | "object" | "string"
                | "symbol" | "undefined" | "unknown" | "void" => {
                    f.write_str("_")?;
                    f.write_str(self.0)
                }
                _ => f.write_str(self.0),
            }
        }
    }

    SafeName(graphql_name)
}

fn maybe_docs<O>(out: &mut O, docs: Option<&str>, indentation: &str) -> fmt::Result
where
    O: fmt::Write,
{
    let Some(docs) = docs else { return Ok(()) };

    writeln!(out, "{indentation}/**")?;

    for line in docs.lines() {
        writeln!(out, "{indentation} * {line}")?;
    }

    writeln!(out, "{indentation} */")
}
