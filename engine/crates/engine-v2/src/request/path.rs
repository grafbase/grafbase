use crate::response::{ResponseKey, ResponseKeys};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryPath(im::Vector<ResponseKey>);

impl IntoIterator for QueryPath {
    type Item = ResponseKey;

    type IntoIter = <im::Vector<ResponseKey> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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

    pub fn iter_strings<'a>(&'a self, keys: &'a ResponseKeys) -> impl Iterator<Item = String> + 'a {
        self.into_iter().map(move |key| keys[*key].to_string())
    }
}
