use async_graphql::{ServerError, dynamic::ResolverContext};

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

impl Resolver for &str {
    fn resolve(&mut self, _: ResolverContext<'_>) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.to_string()))
    }
}

impl Resolver for String {
    fn resolve(&mut self, _: ResolverContext<'_>) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.clone()))
    }
}

impl Resolver for serde_json::Value {
    fn resolve(&mut self, _context: ResolverContext<'_>) -> Option<serde_json::Value> {
        Some(self.clone())
    }
}

impl Resolver for ServerError {
    fn resolve(&mut self, context: ResolverContext<'_>) -> Option<serde_json::Value> {
        context.add_error(self.clone());
        None
    }
}

impl Resolver for Option<serde_json::Value> {
    fn resolve(&mut self, _context: ResolverContext<'_>) -> Option<serde_json::Value> {
        self.clone()
    }
}
