#![allow(unused)]
use wasmtime::component::{ComponentType, Lower};

use crate::state::WasiState;

pub use super::grafbase::sdk::selection_set_resolver_types::*;

impl Host for WasiState {}
