use std::ops::Range;

use id_newtypes::IdRange;
use walker::Walk;

use super::{Field, OperationContext, Selection, SelectionId};

#[derive(Default, Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct SelectionSetRecord(IdRange<SelectionIdSharedVecId>);

impl SelectionSetRecord {
    pub fn empty() -> Self {
        Self(IdRange::empty())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Range<usize>> for SelectionSetRecord {
    fn from(range: Range<usize>) -> Self {
        Self(IdRange::from(range))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(in crate::model) struct SelectionIdSharedVecId(u16);

#[derive(Clone, Copy)]
pub struct SelectionSet<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    ids: IdRange<SelectionIdSharedVecId>,
}

impl<'a> SelectionSet<'a> {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn fields(&self) -> FieldsIterator<'a> {
        FieldsIterator {
            ctx: self.ctx,
            stack: vec![self.ctx.operation[self.ids].iter()],
        }
    }
}

impl<'a> IntoIterator for SelectionSet<'a> {
    type Item = Selection<'a>;
    type IntoIter = SelectionIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        SelectionIterator {
            ctx: self.ctx,
            selection_ids: self.ctx.operation[self.ids].iter(),
        }
    }
}

impl<'a> Walk<OperationContext<'a>> for SelectionSetRecord {
    type Walker<'w>
        = SelectionSet<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SelectionSet {
            ctx: ctx.into(),
            ids: self.0,
        }
    }
}

impl std::fmt::Debug for SelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

pub struct FieldsIterator<'a> {
    ctx: OperationContext<'a>,
    stack: Vec<std::slice::Iter<'a, SelectionId>>,
}

impl<'a> Iterator for FieldsIterator<'a> {
    type Item = Field<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let iter = self.stack.last_mut()?;
            if let Some(next) = iter.next() {
                match next.walk(self.ctx) {
                    Selection::Field(field) => return Some(field),
                    Selection::FragmentSpread(spread) => {
                        self.stack
                            .push(self.ctx.operation[spread.fragment().selection_set_record.0].iter());
                    }
                    Selection::InlineFragment(fragment) => {
                        self.stack
                            .push(self.ctx.operation[fragment.selection_set_record.0].iter());
                    }
                }
            } else {
                self.stack.pop();
            }
        }
    }
}

pub struct SelectionIterator<'a> {
    ctx: OperationContext<'a>,
    selection_ids: std::slice::Iter<'a, SelectionId>,
}

impl<'a> Iterator for SelectionIterator<'a> {
    type Item = Selection<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.selection_ids.next().walk(self.ctx)
    }
}
