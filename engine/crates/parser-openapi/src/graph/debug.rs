use std::fmt::Debug;

use super::OpenApiGraph;

/// std::fmt::Debug for a node in our graph
///
/// Our nodes are usually just wrappers around indices so
/// the default Debug impl is pretty useless.  This trait adds an
/// extra graph parameter so we can get at the actual data.
pub trait DebugNode: Sized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, graph: &OpenApiGraph) -> std::fmt::Result;

    fn debug<'a>(&'a self, graph: &'a OpenApiGraph) -> GraphDebug<'a, Self> {
        GraphDebug(graph, self)
    }
}

impl<Node> DebugNode for Vec<Node>
where
    Node: DebugNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, graph: &OpenApiGraph) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter().map(|node| node.debug(graph)))
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct GraphDebug<'a, Node>(&'a OpenApiGraph, &'a Node);

impl<'a, Node> Debug for GraphDebug<'a, Node>
where
    Node: DebugNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(f, self.0)
    }
}
