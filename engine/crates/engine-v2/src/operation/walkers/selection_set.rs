use schema::{Definition, DefinitionWalker};

use super::{FieldWalker, FragmentSpreadWalker, InlineFragmentWalker, OperationWalker};
use crate::operation::{Selection, SelectionSetId, SelectionSetType};

pub type SelectionSetWalker<'a> = OperationWalker<'a, SelectionSetId, ()>;
pub type SelectionSetTypeWalker<'a> = OperationWalker<'a, SelectionSetType, Definition>;

impl<'a> SelectionSetWalker<'a> {
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

impl PartialEq for SelectionSetTypeWalker<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}
impl Eq for SelectionSetTypeWalker<'_> {}

impl<'a> std::ops::Deref for SelectionSetTypeWalker<'a> {
    type Target = DefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

pub(crate) enum SelectionWalker<'a> {
    Field(FieldWalker<'a>),
    FragmentSpread(FragmentSpreadWalker<'a>),
    InlineFragment(InlineFragmentWalker<'a>),
}

impl<'a> IntoIterator for SelectionSetWalker<'a> {
    type Item = SelectionWalker<'a>;

    type IntoIter = SelectionSetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SelectionSetIterator {
            selection_set: self,
            next_index: 0,
        }
    }
}

pub(crate) struct SelectionSetIterator<'a> {
    selection_set: SelectionSetWalker<'a>,
    next_index: usize,
}

impl<'a> Iterator for SelectionSetIterator<'a> {
    type Item = SelectionWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let selection = self.selection_set.as_ref().items.get(self.next_index)?;
        self.next_index += 1;
        Some(match selection {
            Selection::Field(id) => SelectionWalker::Field(self.selection_set.walk(*id)),
            Selection::FragmentSpread(id) => SelectionWalker::FragmentSpread(self.selection_set.walk(*id)),
            Selection::InlineFragment(id) => SelectionWalker::InlineFragment(self.selection_set.walk(*id)),
        })
    }
}

pub(crate) struct SelectionSetFieldsIterator<'a> {
    selections: Vec<SelectionSetIterator<'a>>,
}

impl<'a> Iterator for SelectionSetFieldsIterator<'a> {
    type Item = FieldWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let selections = self.selections.last_mut()?;
            if let Some(selection) = selections.next() {
                match selection {
                    SelectionWalker::Field(field) => return Some(field),
                    SelectionWalker::InlineFragment(inline) => {
                        self.selections.push(inline.selection_set().into_iter());
                    }
                    SelectionWalker::FragmentSpread(spread) => {
                        self.selections.push(spread.selection_set().into_iter());
                    }
                };
            } else {
                self.selections.pop();
            }
        }
    }
}

impl<'a> std::fmt::Debug for SelectionSetWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field("id", &self.id())
            .field("ty", &self.ty().name())
            .field("items", &self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for SelectionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => field.fmt(f),
            Self::FragmentSpread(spread) => spread.fmt(f),
            Self::InlineFragment(fragment) => fragment.fmt(f),
        }
    }
}
