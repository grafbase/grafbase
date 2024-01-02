use crate::analyze::{AnalyzedSchema, BuiltinScalar, Definition, GraphqlType, ListWrapper, ObjectKind, TypeKind};
use std::fmt;

const INDENT: &str = "  ";
const DOUBLE_INDENT: &str = "    ";

pub(crate) fn generate_module<O>(schema: &AnalyzedSchema<'_>, out: &mut O) -> fmt::Result
where
    O: fmt::Write,
{
    out.write_str(HEADER)?;
    out.write_str("export type Schema = {\n")?;

    for definition in &schema.definitions {
        match definition {
            Definition::Object(id) => {
                let object_type = &schema[*id];
                let mut fields = schema.iter_object_fields(*id).peekable();

                if fields.peek().is_none() {
                    continue;
                }

                let object_type_name = object_type.name;
                let is_input_object = matches!(object_type.kind, ObjectKind::InputObject);

                maybe_docs(out, object_type.docs, INDENT)?;
                writeln!(out, "{INDENT}'{object_type_name}': {{")?;

                if let ObjectKind::Object = object_type.kind {
                    writeln!(out, "{DOUBLE_INDENT}__typename?: '{object_type_name}';")?;
                }

                for field in fields {
                    let field_optional = !is_input_object
                        && (matches!(
                            field.r#type.kind,
                            TypeKind::Definition(Definition::Object(_) | Definition::Union(_))
                        ) || field.resolver_name.is_some()
                            || field.has_arguments);

                    let field_optional = if field_optional { "?" } else { "" };
                    let field_name = field.name;
                    let field_type = render_graphql_type(&field.r#type, schema);

                    maybe_docs(out, field.docs, INDENT)?;
                    writeln!(out, "{DOUBLE_INDENT}{field_name}{field_optional}: {field_type};")?;
                }

                writeln!(out, "{INDENT}}};")?;
            }
            Definition::Union(union_id) => {
                let union_name = schema[*union_id].name;
                write!(out, "{INDENT}'{union_name}':")?;
                for variant in schema.iter_union_variants(*union_id) {
                    write!(out, " | Schema['{}']", variant.name)?;
                }
                out.write_str(";\n")?;
            }
            Definition::Enum(enum_id) => {
                let r#enum = &schema[*enum_id];
                let enum_name = r#enum.name;
                maybe_docs(out, r#enum.docs, INDENT)?;
                write!(out, "{INDENT}'{enum_name}': ")?;
                for variant in schema.iter_enum_variants(*enum_id) {
                    write!(out, "| '{variant}'")?;
                }
                out.write_str(";\n")?;
            }
            Definition::CustomScalar(id) => {
                let scalar = &schema[*id];
                let scalar_name = scalar.name;
                maybe_docs(out, scalar.docs, INDENT)?;
                writeln!(out, "{INDENT}'{scalar_name}': any;")?;
            }
        }
    }

    out.write_str("};\n")?;

    write_resolver_type(schema, out)
}

fn render_graphql_type(r#type: &GraphqlType, schema: &AnalyzedSchema<'_>) -> String {
    let mut type_string = match &r#type.kind {
        TypeKind::BuiltinScalar(scalar) => match scalar {
            BuiltinScalar::Int | BuiltinScalar::Float | BuiltinScalar::BigInt | BuiltinScalar::Timestamp => "number",
            BuiltinScalar::String
            | BuiltinScalar::Id
            | BuiltinScalar::Url
            | BuiltinScalar::Email
            | BuiltinScalar::Date
            | BuiltinScalar::IPAddress
            | BuiltinScalar::PhoneNumber
            | BuiltinScalar::Bytes
            | BuiltinScalar::Decimal
            | BuiltinScalar::DateTime => "string",
            BuiltinScalar::Json => "any",
            BuiltinScalar::Boolean => "boolean",
        }
        .to_owned(),
        TypeKind::Definition(def) => {
            let name = match *def {
                Definition::CustomScalar(id) => schema[id].name,
                Definition::Enum(id) => schema[id].name,
                Definition::Object(id) => schema[id].name,
                Definition::Union(id) => schema[id].name,
            };
            format!("Schema['{name}']")
        }
    };

    if r#type.inner_is_nullable() {
        type_string.push_str(" | null");
    }

    for wrapper in r#type.iter_list_wrappers() {
        type_string = match wrapper {
            ListWrapper::NullableList => format!("Array<{type_string}> | null"),
            ListWrapper::NonNullList => format!("Array<{type_string}>"),
        }
    }

    type_string
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

fn write_resolver_type<O: fmt::Write>(schema: &AnalyzedSchema<'_>, out: &mut O) -> fmt::Result {
    let mut fields = schema
        .iter_fields()
        .filter(|(_, _, field)| field.resolver_name.is_some())
        .peekable();

    if fields.peek().is_none() {
        return Ok(());
    }

    out.write_str("\nimport { ResolverFn } from '@grafbase/sdk'\n\n")?;
    out.write_str("export type Resolver = {\n")?;

    for (object_id, field_id, field) in fields {
        let parent_object = &schema[*object_id];
        let parent_object_type_name = parent_object.name;
        let resolver_id = format!("{parent_object_type_name}.{}", field.name);
        let rendered_field_type = render_graphql_type(&field.r#type, schema);

        let mut arguments = String::from("{ ");

        for arg in schema.iter_field_arguments(field_id) {
            arguments.push_str(arg.name);
            arguments.push_str(": ");
            arguments.push_str(&render_graphql_type(&arg.r#type, schema));
            arguments.push_str(", ");
        }

        arguments.push_str(" }");

        writeln!(
            out,
            "{INDENT}'{resolver_id}': ResolverFn<Schema['{parent_object_type_name}'], {arguments}, {rendered_field_type}>"
        )?;
    }

    writeln!(out, "}}\n")
}

const HEADER: &str = r#"// This is a generated file. It should not be edited manually.
//
// You can decide to commit this file or add it to your `.gitignore`.
//
// By convention, this module is imported as `@grafbase/generated`. To make this syntax possible,
// add a `paths` entry to your `tsconfig.json`.
//
//  "compilerOptions": {
//    "paths": {
//      "@grafbase/generated": ["./grafbase/generated"]
//    }
//  }

"#;
