use super::*;

#[derive(Default)]
pub(crate) struct Keys(BTreeSet<Key>);

impl Subgraphs {
    pub(crate) fn iter_object_keys(
        &self,
        object_id: DefinitionId,
    ) -> impl Iterator<Item = KeyWalker<'_>> {
        self.keys
            .0
            .range(
                Key {
                    object_id,
                    selection_set: SelectionId::MIN,
                    resolvable: false,
                }..Key {
                    object_id,
                    selection_set: SelectionId::MAX,
                    resolvable: true,
                },
            )
            .map(|key| self.walk(key))
    }

    pub(crate) fn push_key(
        &mut self,
        object_id: DefinitionId,
        selection_set: SelectionId,
        resolvable: bool,
    ) {
        self.keys.0.insert(Key {
            object_id,
            selection_set,
            resolvable,
        });
    }
}

/// Corresponds to an `@key` annotation.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub(crate) struct Key {
    /// The object type the key is defined on.
    object_id: DefinitionId,
    selection_set: SelectionId,
    resolvable: bool,
}

pub(crate) type KeyWalker<'a> = Walker<'a, &'a Key>;

impl<'a> KeyWalker<'a> {
    pub(crate) fn fields(&self) -> impl Iterator<Item = SelectionWalker<'a>> {
        self.subgraphs
            .selection_sets
            .children(self.id.selection_set)
            .map(|selection| self.subgraphs.walk(selection))
    }

    pub(crate) fn is_resolvable(&self) -> bool {
        self.id.resolvable
    }
}
