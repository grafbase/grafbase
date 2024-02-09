use schema::{CacheConfig, Definition, DefinitionWalker, Merge};

use super::{BoundFieldWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker, OperationWalker};
use crate::request::{BoundSelection, BoundSelectionSetId, SelectionSetType};

pub type BoundSelectionSetWalker<'a> = OperationWalker<'a, BoundSelectionSetId>;
pub type SelectionSetTypeWalker<'a> = OperationWalker<'a, SelectionSetType, Definition>;

impl<'a> BoundSelectionSetWalker<'a> {
    pub fn ty(&self) -> SelectionSetTypeWalker<'a> {
        let ty = self.as_ref().ty;
        self.walk_with(ty, Definition::from(ty))
    }

    pub fn fields(self) -> SelectionSetFieldsIterator<'a> {
        SelectionSetFieldsIterator {
            selections: vec![self.into_iter()],
        }
    }
}

impl<'a> BoundSelectionSetWalker<'a> {
    // this merely traverses the selection set recursively and merge all cache_config present in the
    // selected fields
    pub fn cache_config(self) -> Option<CacheConfig> {
        self.into_iter()
            .filter_map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    let cache_config = field.schema_field().and_then(|definition| {
                        definition
                            .cache_config()
                            .merge(definition.ty().inner().as_object().and_then(|obj| obj.cache_config()))
                    });
                    let selection_set_cache_config = field
                        .selection_set()
                        .and_then(|selection_set| selection_set.cache_config());
                    cache_config.merge(selection_set_cache_config)
                }
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().cache_config(),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().cache_config(),
            })
            .reduce(|a, b| a.merge(b))
    }
}

impl<'a> std::ops::Deref for SelectionSetTypeWalker<'a> {
    type Target = DefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

pub(crate) enum BoundSelectionWalker<'a> {
    Field(BoundFieldWalker<'a>),
    FragmentSpread(BoundFragmentSpreadWalker<'a>),
    InlineFragment(BoundInlineFragmentWalker<'a>),
}

impl<'a> IntoIterator for BoundSelectionSetWalker<'a> {
    type Item = BoundSelectionWalker<'a>;

    type IntoIter = SelectionSetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SelectionSetIterator {
            selection_set: self,
            next_index: 0,
        }
    }
}

pub(crate) struct SelectionSetIterator<'a> {
    selection_set: BoundSelectionSetWalker<'a>,
    next_index: usize,
}

impl<'a> Iterator for SelectionSetIterator<'a> {
    type Item = BoundSelectionWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let selection = self.selection_set.as_ref().items.get(self.next_index)?;
        self.next_index += 1;
        Some(match selection {
            BoundSelection::Field(id) => BoundSelectionWalker::Field(self.selection_set.walk(*id)),
            BoundSelection::FragmentSpread(id) => BoundSelectionWalker::FragmentSpread(self.selection_set.walk(*id)),
            BoundSelection::InlineFragment(id) => BoundSelectionWalker::InlineFragment(self.selection_set.walk(*id)),
        })
    }
}

pub(crate) struct SelectionSetFieldsIterator<'a> {
    selections: Vec<SelectionSetIterator<'a>>,
}

impl<'a> Iterator for SelectionSetFieldsIterator<'a> {
    type Item = BoundFieldWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let selections = self.selections.last_mut()?;
            if let Some(selection) = selections.next() {
                match selection {
                    BoundSelectionWalker::Field(field) => return Some(field),
                    BoundSelectionWalker::InlineFragment(inline) => {
                        self.selections.push(inline.selection_set().into_iter());
                    }
                    BoundSelectionWalker::FragmentSpread(spread) => {
                        self.selections.push(spread.selection_set().into_iter());
                    }
                };
            } else {
                self.selections.pop();
            }
        }
    }
}

impl<'a> std::fmt::Debug for BoundSelectionSetWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSet")
            .field("id", &self.id())
            .field("ty", &self.ty().name())
            .field("items", &self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundSelectionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => field.fmt(f),
            Self::FragmentSpread(spread) => spread.fmt(f),
            Self::InlineFragment(fragment) => fragment.fmt(f),
        }
    }
}
