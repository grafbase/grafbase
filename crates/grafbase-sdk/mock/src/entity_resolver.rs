use async_graphql::{ServerError, dynamic::ResolverContext};

pub struct EntityResolverContext<'a> {
    pub inner_context: &'a ResolverContext<'a>,
    pub typename: String,
    pub representation: serde_json::Map<String, serde_json::Value>,
}

impl<'a> EntityResolverContext<'a> {
    pub(super) fn new(inner_context: &'a ResolverContext<'a>, representation: serde_json::Value) -> Self {
        let serde_json::Value::Object(representation) = representation else {
            panic!("repesentations need to be objects");
        };

        let typename = representation["__typename"]
            .as_str()
            .expect("a representation must have __typename")
            .into();

        EntityResolverContext {
            inner_context,
            typename,
            representation,
        }
    }

    pub fn add_error(&self, error: ServerError) {
        self.inner_context.query_env.errors.lock().unwrap().push(error);
    }
}

pub trait EntityResolver: Send + Sync {
    fn resolve(&mut self, context: EntityResolverContext<'_>) -> Option<serde_json::Value>;
}

impl<F> EntityResolver for F
where
    for<'a> F: FnMut(EntityResolverContext<'a>) -> Option<serde_json::Value> + Send + Sync,
{
    fn resolve(&mut self, context: EntityResolverContext<'_>) -> Option<serde_json::Value> {
        self(context)
    }
}

impl EntityResolver for serde_json::Value {
    fn resolve(&mut self, _context: EntityResolverContext<'_>) -> Option<serde_json::Value> {
        Some(self.clone())
    }
}

impl EntityResolver for ServerError {
    fn resolve(&mut self, context: EntityResolverContext<'_>) -> Option<serde_json::Value> {
        context.add_error(self.clone());
        None
    }
}

impl EntityResolver for Option<serde_json::Value> {
    fn resolve(&mut self, _context: EntityResolverContext<'_>) -> Option<serde_json::Value> {
        self.clone()
    }
}
