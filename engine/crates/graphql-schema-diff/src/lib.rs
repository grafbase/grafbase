#![doc = include_str!("../README.md")]
#![allow(unused_crate_dependencies)]
#![deny(missing_docs)]

mod change;
mod state;
mod traverse_schemas;

pub use change::{Change, ChangeKind};

use self::state::*;
use async_graphql_parser::{types as ast, Positioned};
use async_graphql_value::ConstValue;
use std::collections::HashMap;

/// Diff two GraphQL schemas.
pub fn diff(source: &str, target: &str) -> Result<Vec<Change>, async_graphql_parser::Error> {
    let [source, target] = [source, target].map(|sdl| -> Result<_, async_graphql_parser::Error> {
        if sdl.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(async_graphql_parser::parse_schema(sdl)?))
        }
    });
    let [source, target] = [source?, target?];

    let mut state = DiffState::default();

    traverse_schemas::traverse_schemas([source.as_ref(), target.as_ref()], &mut state);

    Ok(state.into_changes())
}
