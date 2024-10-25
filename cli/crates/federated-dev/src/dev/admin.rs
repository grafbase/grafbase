mod types;

pub(crate) use self::types::{Header, PublishSubgraphInput};
use super::bus::AdminBus;
pub(crate) struct MutationRoot;

use async_graphql::{Context, Error, Object};

#[Object]
impl MutationRoot {
    pub(crate) async fn publish_subgraph(&self, ctx: &Context<'_>, input: PublishSubgraphInput) -> Result<bool, Error> {
        log::trace!("publishing a new subgraph");

        let bus = ctx.data::<AdminBus>().expect("must be a bus");
        let schema = bus
            .introspect_schema(&input.name, input.url.clone(), input.headers.clone())
            .await?;

        bus.compose_graph(input.name, input.url, input.headers, schema).await?;

        Ok(true)
    }
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    pub(crate) async fn ping(&self) -> bool {
        true
    }
}
