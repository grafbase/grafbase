use super::{BoundResponseKey, ResponseEdge, UnpackedResponseEdge};

#[derive(Default, Debug, Clone)]
pub struct ResponsePath(Vec<ResponseEdge>);

impl ResponsePath {
    pub fn child(&self, segment: impl Into<ResponseEdge>) -> ResponsePath {
        let mut path = self.0.clone();
        path.push(segment.into());
        ResponsePath(path)
    }

    pub fn push(&mut self, edge: ResponseEdge) {
        self.0.push(edge);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResponseEdge> {
        self.0.iter()
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
