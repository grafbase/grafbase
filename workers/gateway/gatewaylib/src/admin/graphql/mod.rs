mod cache;

#[derive(Debug, async_graphql::MergedObject, Default)]
pub struct Query;

#[derive(Debug, async_graphql::MergedObject, Default)]
pub struct Mutation(cache::CachePurgeMutation);
