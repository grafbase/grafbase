use async_graphql::dynamic::ResolverContext;

pub trait Resolver: Send + Sync {
    fn resolve(&mut self, context: ResolverContext<'_>) -> Option<serde_json::Value>;
}

impl<F> Resolver for F
where
    for<'a> F: FnMut(ResolverContext<'a>) -> Option<serde_json::Value> + Send + Sync,
{
    fn resolve(&mut self, context: ResolverContext<'_>) -> Option<serde_json::Value> {
        self(context)
    }
}
