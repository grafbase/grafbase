use crate::{RequiredScopesId, SchemaWalker, StringId};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct RequiredScopes(Vec<Vec<StringId>>);

impl RequiredScopes {
    pub fn new(mut scopes: Vec<Vec<StringId>>) -> Self {
        for scopes in &mut scopes {
            scopes.sort_unstable();
        }
        scopes.sort_unstable();
        Self(scopes)
    }
}

pub type RequiredScopesWalker<'a> = SchemaWalker<'a, RequiredScopesId>;

impl RequiredScopesWalker<'_> {
    pub fn matches(&self, scopes: &[&str]) -> bool {
        self.as_ref()
            .0
            .iter()
            .any(|any_of| any_of.iter().all(|id| scopes.contains(&self.schema[*id].as_str())))
    }
}
