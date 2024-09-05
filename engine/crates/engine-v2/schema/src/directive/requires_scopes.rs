use readable::{Iter, Readable};

use crate::{Schema, StringId, MAX_ID};

#[derive(Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequiresScopesDirectiveRecord {
    scope_ids: Vec<Vec<StringId>>,
}

impl RequiresScopesDirectiveRecord {
    pub fn new(mut scope_ids: Vec<Vec<StringId>>) -> Self {
        for scopes in &mut scope_ids {
            scopes.sort_unstable();
        }
        scope_ids.sort_unstable();
        Self { scope_ids }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct RequiresScopesDirectiveId(std::num::NonZero<u32>);

/// An index into the outer vector of an instance of RequiredScopes
///
/// Used to record which of the sets of scopes matched in a given requests
#[derive(Clone, Copy, Debug)]
pub struct RequiresScopeSetIndex(u16);

#[derive(Clone, Copy)]
pub struct RequiresScopesDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: RequiresScopesDirectiveId,
}

impl<'a> RequiresScopesDirective<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a RequiresScopesDirectiveRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> RequiresScopesDirectiveId {
        self.id
    }

    pub fn scopes(&self) -> impl Iter<Item: Iter<Item = &'a str> + 'a> + 'a {
        let schema = self.schema;
        self.as_ref()
            .scope_ids
            .iter()
            .map(move |items| items.iter().map(move |id| schema[*id].as_ref()))
    }

    pub fn get_scopes(&self, index: RequiresScopeSetIndex) -> impl ExactSizeIterator<Item = &'a str> {
        self.as_ref().scope_ids[index.0 as usize].read(self.schema)
    }

    pub fn matches(&self, scopes: &[&str]) -> Option<RequiresScopeSetIndex> {
        self.scopes().enumerate().find_map(|(index, mut required_scopes)| {
            if required_scopes.all(|scope| scopes.contains(&scope)) {
                Some(RequiresScopeSetIndex(index as u16))
            } else {
                None
            }
        })
    }
}

impl Readable<Schema> for RequiresScopesDirectiveId {
    type Reader<'a> = RequiresScopesDirective<'a>;
    fn read<'s>(self, schema: &'s Schema) -> Self::Reader<'s>
    where
        Self: 's,
    {
        RequiresScopesDirective { schema, id: self }
    }
}

impl std::fmt::Debug for RequiresScopesDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredScopesDirective")
            .field(
                "scopes",
                &self.scopes().map(|items| items.collect::<Vec<_>>()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
