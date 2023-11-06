#![deny(unsafe_code, missing_docs, rust_2018_idioms)]
#![allow(unused_crate_dependencies)]
#![doc = include_str!("../README.md")]

mod context;
mod diagnostics;
mod validate;

use self::{context::*, diagnostics::*};
use std::collections::HashMap;

/// Validate the GraphQL SDL document and produce a possibly empty collection of errors.
pub fn validate(sdl: &str) -> Diagnostics {
    let parsed_ast = match async_graphql_parser::parse_schema(sdl) {
        Ok(ast) => ast,
        Err(err) => {
            return Diagnostics {
                errors: vec![miette::miette! {
                    "Syntax error: {}",
                    err.to_string()
                }],
            };
        }
    };

    let mut ctx = Context::new(
        sdl,
        HashMap::with_capacity(parsed_ast.definitions.len()),
        Diagnostics::default(),
    );

    validate::validate(&parsed_ast, &mut ctx);

    ctx.diagnostics
}
