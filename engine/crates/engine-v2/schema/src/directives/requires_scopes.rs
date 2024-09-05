use crate::{RequiredScopesId, SchemaWalker, StringId};

#[derive(Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequiredScopesRecord(Vec<Vec<StringId>>);

/// An index into the outer vector of an instance of RequiredScopes
///
/// Used to record which of the sets of scopes matched in a given requests
#[derive(Clone, Copy, Debug)]
pub struct RequiredScopeSetIndex(u16);

impl RequiredScopesRecord {
    pub fn new(mut scopes: Vec<Vec<StringId>>) -> Self {
        for scopes in &mut scopes {
            scopes.sort_unstable();
        }
        scopes.sort_unstable();
        Self(scopes)
    }
}

pub type RequiredScopesWalker<'a> = SchemaWalker<'a, RequiredScopesId>;

impl<'a> RequiredScopesWalker<'a> {
    pub fn scopes(&self, index: RequiredScopeSetIndex) -> impl ExactSizeIterator<Item = &'a str> {
        self.as_ref().0[index.0 as usize]
            .iter()
            .map(|id| self.schema[*id].as_ref())
    }

    pub fn matches(&self, scopes: &[&str]) -> Option<RequiredScopeSetIndex> {
        self.as_ref()
            .0
            .iter()
            .enumerate()
            .find(|(_, any_of)| any_of.iter().all(|id| scopes.contains(&self.schema[*id].as_str())))
            .map(|(index, _)| RequiredScopeSetIndex(index as u16))
    }
}
