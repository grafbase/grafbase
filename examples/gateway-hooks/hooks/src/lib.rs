#[allow(warnings)]
mod bindings;
mod common;
use common::*;

// Individual interface implementations
mod authorization;
mod gateway;

struct Component;

bindings::export!(Component with_types_in bindings);
