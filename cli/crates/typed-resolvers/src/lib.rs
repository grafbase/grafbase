#![allow(unused_crate_dependencies)]

mod analyze;
mod check_resolver;
mod codegen;
mod error;

pub use self::analyze::{analyze as analyze_schema, AnalyzedSchema};
pub use engine_parser::parse_schema;

use self::error::CodegenError;
use std::{ffi, fmt, path::Path};

/// Generate a TypeScript module that contains input and output type definitions for resolver
/// authoring purposes, based on the passed in SDL schema.
pub fn generate_ts_resolver_types<O>(graphql_sdl: &str, out: &mut O) -> Result<(), CodegenError>
where
    O: fmt::Write,
{
    let parsed_schema = parse_schema::<&str>(graphql_sdl)?;

    if experimental_codegen_is_enabled(&parsed_schema) {
        let analyzed_schema = analyze::analyze(&parsed_schema);
        codegen::generate_module(&analyzed_schema, out)?;
        Ok(())
    } else {
        Err(CodegenError::ExperimentalFeatureNotEnabled)
    }
}

#[must_use]
pub struct AnalyzedResolvers {
    pub errs: Vec<miette::Error>,
}

pub fn check_resolver(sdl: &str, resolver_path: &Path) -> miette::Result<()> {
    if resolver_path.extension() != Some(ffi::OsStr::new("ts")) {
        return Ok(());
    }
    let Ok(parsed_schema) = parse_schema::<&str>(sdl) else {
        return Ok(());
    };
    let schema = analyze::analyze(&parsed_schema);

    check_resolver::check_resolver(resolver_path, &schema)
}

/// Returns either a GraphQL SDL string that defines the resolvers as type extensions, or errors.
pub fn check_resolvers(resolvers_root: &Path, schema: &analyze::AnalyzedSchema<'_>) -> AnalyzedResolvers {
    let mut errs = Vec::new();

    for entry in walkdir::WalkDir::new(resolvers_root).into_iter().filter_map(Result::ok) {
        if entry.path().extension() != Some(ffi::OsStr::new("ts")) {
            continue;
        }

        if let Err(err) = check_resolver::check_resolver(entry.path(), schema) {
            errs.push(err);
        }
    }

    AnalyzedResolvers { errs }
}

fn experimental_codegen_is_enabled(parsed_schema: &engine_parser::types::ServiceDocument) -> bool {
    const EXPERIMENTAL_DIRECTIVE: &str = "experimental";
    const CODEGEN_ARGUMENT: &str = "codegen";

    parsed_schema
        .definitions
        .iter()
        .filter_map(|def| match def {
            engine_parser::types::TypeSystemDefinition::Schema(schema_definition) => Some(schema_definition),
            _ => None,
        })
        .flat_map(|def| def.node.directives.iter())
        .any(|directive| {
            directive.node.name.as_str() == EXPERIMENTAL_DIRECTIVE
                && directive.node.arguments.iter().any(|(name, value)| {
                    name.node == CODEGEN_ARGUMENT && matches!(value.node, engine_value::ConstValue::Boolean(true))
                })
        })
}
