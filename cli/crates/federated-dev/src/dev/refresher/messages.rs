use actix::{Addr, Message};

use crate::{
    admin::PublishSubgraphError,
    composer::{Composer, Subgraph},
};

pub(crate) struct RefreshGraphs {
    graphs: Vec<(String, Subgraph)>,
    composer: Addr<Composer>,
}

impl RefreshGraphs {
    pub(crate) fn new(composer: Addr<Composer>, graphs: impl IntoIterator<Item = (String, Subgraph)>) -> Self {
        Self {
            graphs: Vec::from_iter(graphs),
            composer,
        }
    }

    pub(crate) fn into_parts(self) -> (Vec<(String, Subgraph)>, Addr<Composer>) {
        (self.graphs, self.composer)
    }
}

impl Message for RefreshGraphs {
    type Result = Result<(), PublishSubgraphError>;
}
