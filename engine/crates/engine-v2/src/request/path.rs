use crate::response::ResponseKey;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryPath(im::Vector<ResponseKey>);

impl<'a> IntoIterator for &'a QueryPath {
    type Item = &'a ResponseKey;

    type IntoIter = <&'a im::Vector<ResponseKey> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl QueryPath {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn child(&self, id: ResponseKey) -> Self {
        let mut child = self.clone();
        child.0.push_back(id);
        child
    }
}
