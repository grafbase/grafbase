#![allow(unused_crate_dependencies)]

mod analyze;
mod check_resolver;
mod codegen;
mod error;

pub use self::{
    analyze::{analyze as analyze_schema, AnalyzedSchema},
    check_resolver::check_resolver,
};
pub use engine_parser::parse_schema;

use self::error::CodegenError;
use std::{ffi, fmt, path::Path};

pub struct CustomResolver {
    pub parent_type_name: String,
    pub field_name: String,
    pub resolver_name: String,
}

/// Generate a TypeScript module that contains input and output type definitions for resolver
/// authoring purposes, based on the passed in SDL schema.
pub fn generate_ts_resolver_types<O>(schema: &analyze::AnalyzedSchema<'_>, out: &mut O) -> Result<(), CodegenError>
where
    O: fmt::Write,
{
    Ok(codegen::generate_module(schema, out)?)
}

#[must_use]
pub struct AnalyzedResolvers {
    pub errs: Vec<miette::Error>,
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
