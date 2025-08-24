use cynic_parser::Span;

use crate::SubgraphId;

#[derive(Debug, Clone)]
pub(crate) struct CachedJoinTypeDirective<'sdl> {
    pub subgraph_id: SubgraphId,
    pub key: Option<&'sdl str>,
    pub resolvable: bool,
    pub is_interface_object: bool,
    pub arguments_span: Span,
}
