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
    let source = async_graphql_parser::parse_schema(source)?;
    let target = async_graphql_parser::parse_schema(target)?;

    let mut state = DiffState::default();

    traverse_schemas::traverse_schemas([&source, &target], &mut state);

    Ok(state.into_changes())
}
