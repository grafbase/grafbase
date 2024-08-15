#![doc = include_str!("../README.md")]
#![allow(unused_crate_dependencies)]
#![deny(missing_docs)]

mod change;
mod state;
mod traverse_schemas;

pub use change::{Change, ChangeKind, Span};

use self::state::*;
use cynic_parser::type_system as ast;
use std::collections::HashMap;

/// Diff two GraphQL schemas.
pub fn diff(source: &str, target: &str) -> Result<Vec<Change>, cynic_parser::Error> {
    let [source, target] = [source, target].map(|sdl| -> Result<_, cynic_parser::Error> {
        if sdl.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(cynic_parser::parse_type_system_document(sdl)?))
        }
    });
    let [source, target] = [source?, target?];

    let mut state = DiffState::default();

    traverse_schemas::traverse_schemas([source.as_ref(), target.as_ref()], &mut state);

    Ok(state.into_changes())
}
