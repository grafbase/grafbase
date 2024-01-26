use std::collections::VecDeque;

use schema::{CacheConfig, Definition, DefinitionWalker, Merge};

use super::{
    BoundFieldWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker, ExecutorWalkContext, OperationWalker,
};
use crate::request::{BoundSelection, BoundSelectionSetId, SelectionSetType};

pub type BoundSelectionSetWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, BoundSelectionSetId, (), CtxOrUnit>;
pub type SelectionSetTypeWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, SelectionSetType, Definition, CtxOrUnit>;

impl<'a, C> BoundSelectionSetWalker<'a, C> {
    pub fn ty(&self) -> SelectionSetTypeWalker<'a, ()> {
        let ty = self.as_ref().ty;
        self.without_ctx().walk_with(ty, Definition::from(ty))
    }
}

impl<'a> BoundSelectionSetWalker<'a, ()> {
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

impl<'a, C> std::ops::Deref for SelectionSetTypeWalker<'a, C> {
    type Target = DefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

pub enum BoundSelectionWalker<'a, C = ()> {
    Field(BoundFieldWalker<'a, C>),
    FragmentSpread(BoundFragmentSpreadWalker<'a, C>),
    InlineFragment(BoundInlineFragmentWalker<'a, C>),
}

impl<'a, C: Copy> IntoIterator for BoundSelectionSetWalker<'a, C> {
    type Item = BoundSelectionWalker<'a, C>;

    type IntoIter = SelectionIterator<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        SelectionIterator {
            walker: self.walk(()),
            selections: self.operation[self.item].items.iter().collect(),
        }
    }
}

pub struct SelectionIterator<'a, C> {
    walker: OperationWalker<'a, (), (), C>,
    selections: VecDeque<&'a BoundSelection>,
}

impl<'a, C: Copy> Iterator for SelectionIterator<'a, C> {
    type Item = BoundSelectionWalker<'a, C>;

    fn next(&mut self) -> Option<Self::Item> {
        let selection = self.selections.pop_front()?;
        Some(match selection {
            BoundSelection::Field(id) => BoundSelectionWalker::Field(self.walker.walk(*id)),
            BoundSelection::FragmentSpread(id) => BoundSelectionWalker::FragmentSpread(self.walker.walk(*id)),
            BoundSelection::InlineFragment(id) => BoundSelectionWalker::InlineFragment(self.walker.walk(*id)),
        })
    }
}

impl<'a> std::fmt::Debug for BoundSelectionSetWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSet")
            .field("id", &self.id())
            .field("ty", &self.ty().name())
            .field("items", &self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundSelectionSetWalker<'a, ExecutorWalkContext<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSet")
            .field("id", &self.id())
            .field("ty", &self.ty().name())
            .field("items", &self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundSelectionWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => field.fmt(f),
            Self::FragmentSpread(spread) => spread.fmt(f),
            Self::InlineFragment(fragment) => fragment.fmt(f),
        }
    }
}

impl<'a> std::fmt::Debug for BoundSelectionWalker<'a, ExecutorWalkContext<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => field.fmt(f),
            Self::FragmentSpread(spread) => spread.fmt(f),
            Self::InlineFragment(fragment) => fragment.fmt(f),
        }
    }
}
