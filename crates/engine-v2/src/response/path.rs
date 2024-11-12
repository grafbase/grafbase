use super::{BoundResponseKey, ResponseEdge, UnpackedResponseEdge};

#[derive(Default, Debug, Clone)]
pub struct ResponsePath(Vec<ResponseEdge>);

impl ResponsePath {
    pub fn child(&self, edge: impl Into<ResponseEdge>) -> ResponsePath {
        let mut path = self.clone();
        path.push(edge);
        path
    }

    pub fn push(&mut self, edge: impl Into<ResponseEdge>) {
        self.0.push(edge.into());
    }
}

impl std::ops::Deref for ResponsePath {
    type Target = [ResponseEdge];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<ResponseEdge>> for ResponsePath {
    fn from(value: Vec<ResponseEdge>) -> Self {
        ResponsePath(value)
    }
}

impl From<BoundResponseKey> for ResponseEdge {
    fn from(value: BoundResponseKey) -> Self {
        UnpackedResponseEdge::BoundResponseKey(value).pack()
    }
}

impl From<usize> for ResponseEdge {
    #[allow(clippy::panic)]
    fn from(index: usize) -> Self {
        UnpackedResponseEdge::Index(index).pack()
    }
}
