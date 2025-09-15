use super::*;

#[derive(Default)]
pub(crate) struct SchemaRoots {
    pub(crate) query: Option<ObjectId>,
    pub(crate) mutation: Option<ObjectId>,
    pub(crate) subscription: Option<ObjectId>,
}
