#![allow(unused_crate_dependencies)]

mod analyze;
mod codegen;
mod error;

use self::error::CodegenError;
use std::fmt;

/// Generate a TypeScript module that contains input and output type definitions for resolver
/// authoring purposes, based on the passed in SDL schema.
pub fn generate_ts_resolver_types<O>(graphql_sdl: &str, out: &mut O) -> Result<(), CodegenError>
where
    O: fmt::Write,
{
    let parsed_schema = graphql_parser::parse_schema::<&str>(graphql_sdl)?;
    let analyzed_schema = analyze::analyze(&parsed_schema);
    codegen::generate_module(&analyzed_schema, out)?;
    Ok(())
}
