use super::*;
use crate::{federated_graph::*, FederatedGraphV3};
use std::fmt;

/// Render a GraphQL SDL string for a federated graph. It does not include any
/// federation-specific directives, it only reflects the final API schema as visible
/// for consumers.
pub fn render_api_sdl(_graph: &FederatedGraphV3) -> Result<String, fmt::Error> {
    let mut sdl = String::new();
    sdl.push('.');
    Ok(sdl)
}
