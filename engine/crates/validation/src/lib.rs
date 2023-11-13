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
    validate_with_options(sdl, Options::default())
}

/// Validate the GraphQL SDL document and produce a possibly empty collection of errors.
pub fn validate_with_options(sdl: &str, options: Options) -> Diagnostics {
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
        options,
    );

    validate::validate(&parsed_ast, &mut ctx);

    ctx.diagnostics
}

bitflags::bitflags! {
    /// Options to configure validation.
    #[derive(Default)]
    pub struct Options: u8 {
        /// If included, this flag enables the validation checking that any type extension extends
        /// a type defined in the same document.
        const FORBID_EXTENDING_UNKNOWN_TYPES = 0b1;
        /// Include validations that are in the current spec draft but not included or not relevant
        /// in the 2021 edition of the spec.
        const DRAFT_VALIDATIONS = 0b01;
    }
}
