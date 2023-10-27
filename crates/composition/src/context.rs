use crate::{Diagnostics, Subgraphs, Supergraph};

pub(crate) struct Context<'a> {
    pub(crate) subgraphs: &'a Subgraphs,
    pub(crate) supergraph: Supergraph,
    pub(crate) diagnostics: Diagnostics,
}
