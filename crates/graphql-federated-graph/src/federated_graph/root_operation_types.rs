use crate::ObjectId;

#[derive(Clone, Debug, Default)]
pub struct RootOperationTypes {
    pub query: Option<ObjectId>,
    pub mutation: Option<ObjectId>,
    pub subscription: Option<ObjectId>,
}
