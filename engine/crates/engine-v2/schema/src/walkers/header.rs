use crate::{HeaderId, HeaderValue, SchemaWalker};

pub type HeaderWalker<'a> = SchemaWalker<'a, HeaderId>;

impl<'a> HeaderWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name]
    }

    pub fn value(&self) -> HeaderValueRef<'a> {
        match self.as_ref().value {
            HeaderValue::Forward(id) => HeaderValueRef::Forward(&self.schema[id]),
            HeaderValue::Static(id) => HeaderValueRef::Static(&self.schema[id]),
        }
    }
}

#[derive(Debug)]
pub enum HeaderValueRef<'a> {
    Forward(&'a str),
    Static(&'a str),
}

impl<'a> std::fmt::Debug for HeaderWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubgraphHeaderWalker")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
    }
}
