#![allow(unused)]
use wasmtime::component::{ComponentType, Lower};

use crate::state::InstanceState;

pub use super::grafbase::sdk::selection_set_resolver_types::*;

impl Host for InstanceState {}
