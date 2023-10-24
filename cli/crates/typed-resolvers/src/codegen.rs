use crate::analyze::{AnalyzedSchema, BuiltinScalar, Definition, GraphqlType, ListWrapper, ObjectKind, TypeKind};
use std::fmt;

const INDENT: &str = "  ";

pub(crate) fn generate_module<O>(schema: &AnalyzedSchema<'_>, out: &mut O) -> fmt::Result
where
    O: fmt::Write,
{
    out.write_str(HEADER)?;

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
                            field.r#type.kind,
                            TypeKind::Definition(Definition::Object(_) | Definition::Union(_))
                        );

                    let field_optional = if field_optional { "?" } else { "" };
                    let field_name = field.name;
                    let field_type = render_graphql_type(&field.r#type, schema);

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

        if idx + 1 < schema.definitions.len() {
            writeln!(out)?;
        }
    }

    write_resolver_type(schema, out)
}

fn render_graphql_type(r#type: &GraphqlType, schema: &AnalyzedSchema<'_>) -> String {
    let mut type_string = match &r#type.kind {
        TypeKind::BuiltinScalar(scalar) => match scalar {
            BuiltinScalar::Int | BuiltinScalar::Float => "number",
            BuiltinScalar::String | BuiltinScalar::Id => "string",
            BuiltinScalar::Boolean => "boolean",
        }
        .to_owned(),
        TypeKind::Definition(def) => safe_ts_type_name(match *def {
            Definition::CustomScalar(id) => schema[id].name,
            Definition::Enum(id) => schema[id].name,
            Definition::Object(id) => schema[id].name,
            Definition::Union(id) => schema[id].name,
        })
        .to_string(),
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

fn write_resolver_type<O: fmt::Write>(schema: &AnalyzedSchema<'_>, out: &mut O) -> fmt::Result {
    let mut fields = schema
        .iter_fields()
        .filter(|(_, _, field)| field.resolver_name.is_some())
        .peekable();

    if fields.peek().is_none() {
        return Ok(());
    }

    writeln!(out, "\nimport * as sdk from '@grafbase/sdk'\n")?;
    writeln!(out, "export type Resolver = {{")?;

    for (object_id, field_id, field) in fields {
        let parent_object = &schema[*object_id];
        let resolver_id = format!("{}.{}", parent_object.name, field.name);
        let parent_object_generated_type_name = safe_ts_type_name(parent_object.name);
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
            "{INDENT}'{resolver_id}': sdk.ResolverFn<{parent_object_generated_type_name}, {arguments}, {rendered_field_type}>"
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
