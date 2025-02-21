//! A module for performing JQ filter selections on JSON data.
//!
//! Enable feature `jq-selection` to use this module.
//!
//! This module provides functionality to parse, compile and execute JQ filter
//! expressions against JSON data. It internally caches compiled filters to avoid
//! recompilation overhead.
//!
//! # Examples
//!
//! ```rust
//! # use grafbase_sed::jq_selection::JqSelection;
//!
//! let mut jq = JqSelection::new();
//! let data = serde_json::json!({"name": "Jane", "age": 25});
//! let results = jq.select(".name", data).unwrap();
//!
//! assert_eq!(results, serde_json::json!("Jane"));
//! ```

use std::iter::Empty;

use core::hash::BuildHasher;
use hashbrown::{hash_table::Entry, DefaultHashBuilder, HashTable};
use jaq_core::{
    load::{Arena, File, Loader},
    Compiler, Ctx, Filter, Native, RcIter,
};
use jaq_json::Val;

/// A struct that holds JQ filter selections
///
/// Use it to select data from a JSON object using JQ syntax. Caches the previously compiled filters,
/// and reuses them to avoid recompiling the same filter multiple times.
///
/// You are supposed to store this struct in your extension and reuse it across multiple requests.
pub struct JqSelection {
    arena: Arena,
    // (╯° · °)╯︵ ┻━┻
    inputs: RcIter<Empty<Result<Val, String>>>,
    // ┬┴┬┴┤
    // ┬┴┬┴┤ ͡°)
    // ┬┴┬┴┤ ͜ʖ ͡°)
    // ┬┴┬┴┤ ͡° ͜ʖ ͡°)
    // ┬┴┬┴┤ ͡° ͜ʖ ͡~)
    // ┬┴┬┴┤ ͡° ͜ʖ ͡°)
    // ┬┴┬┴┤ ͜ʖ ͡°)
    // ┬┴┬┴┤ ͡°)
    // ┬┴┬┴┤
    selection_cache: HashTable<(String, usize)>,
    filters: Vec<Filter<Native<Val>>>,
}

impl Default for JqSelection {
    fn default() -> Self {
        Self {
            arena: Arena::default(),
            inputs: RcIter::new(core::iter::empty()),
            selection_cache: HashTable::new(),
            filters: Vec::new(),
        }
    }
}

impl JqSelection {
    /// Creates a new instance of [`JqSelection`].
    ///
    /// Creates an empty cache of compiled filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects data from a JSON value using a JQ filter.
    ///
    /// This method takes a JQ selection filter string and a JSON value, applies the
    /// filter, and returns an iterator of the results. The filter is compiled and cached
    /// for reuse on subsequent calls with the same filter string.
    pub fn select(
        &mut self,
        selection: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<impl Iterator<Item = anyhow::Result<serde_json::Value>> + '_> {
        let hasher = DefaultHashBuilder::default();
        let hash = hasher.hash_one(selection);
        let hasher = |val: &(String, usize)| hasher.hash_one(&val.0);

        let idx = match self
            .selection_cache
            .entry(hash, |(key, _)| key.as_str() == selection, hasher)
        {
            Entry::Occupied(entry) => entry.get().1,
            Entry::Vacant(vacant_entry) => {
                let program = File {
                    code: selection,
                    path: (),
                };

                let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));

                let modules = loader.load(&self.arena, program).map_err(|e| {
                    let error = e.first().map(|e| e.0.code).unwrap_or_default();
                    anyhow::anyhow!("The selection is not valid jq syntax: `{error}`")
                })?;

                let filter = Compiler::default()
                    .with_funs(jaq_std::funs().chain(jaq_json::funs()))
                    .compile(modules)
                    .map_err(|e| {
                        let error = e.first().map(|e| e.0.code).unwrap_or_default();
                        anyhow::anyhow!("The selection is not valid jq syntax: `{error}`")
                    })?;

                self.filters.push(filter);

                let index = self.filters.len() - 1;
                vacant_entry.insert((selection.to_string(), index));

                index
            }
        };

        let filter = &self.filters[idx];
        let filtered = filter.run((Ctx::new([], &self.inputs), Val::from(data)));

        Ok(filtered.map(|v| match v {
            Ok(val) => Ok(serde_json::Value::from(val)),
            Err(e) => Err(anyhow::anyhow!("{e}")),
        }))
    }
}
