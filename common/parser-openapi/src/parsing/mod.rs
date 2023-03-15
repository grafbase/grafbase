use std::collections::HashMap;

use petgraph::{graph::NodeIndex, Graph};

use crate::{
    graph::{Edge, Node},
    Error,
};

use self::components::{Components, Ref};

pub mod components;
mod graph;
mod grouping;
pub mod operations;

#[derive(Default)]
pub struct Context {
    pub graph: Graph<Node, Edge>,
    schema_index: HashMap<Ref, NodeIndex>,
    pub operation_indices: Vec<NodeIndex>,
    errors: Vec<Error>,
}

pub fn parse(spec: openapiv3::OpenAPI) -> Result<Context, Vec<Error>> {
    let mut ctx = Context::default();

    let mut components = Components::default();
    if let Some(spec_components) = &spec.components {
        components.extend(&mut ctx, spec_components);
        graph::extract_components(&mut ctx, spec_components);
    }

    graph::extract_operations(&mut ctx, &spec.paths, components);
    grouping::determine_resource_relationships(&mut ctx);

    if ctx.errors.is_empty() {
        Ok(ctx)
    } else {
        Err(ctx.errors)
    }
}
