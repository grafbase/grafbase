use std::ops::Range;

use walker::{Walk, WalkIterator};

// Not necessary anymore when Rust stabilize std::iter::Step
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct IdRange<Id: Copy> {
    pub start: Id,
    pub end: Id,
}

impl<Id> Default for IdRange<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    fn default() -> Self {
        Self {
            start: Id::from(0),
            end: Id::from(0),
        }
    }
}

impl<Id> IdRange<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        usize::from(self.end) - usize::from(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, i: usize) -> Option<Id> {
        let i = usize::from(self.start) + i;
        if i < usize::from(self.end) {
            Some(Id::from(i))
        } else {
            None
        }
    }

    pub fn index_of(&self, id: Id) -> Option<usize> {
        let id = usize::from(id);
        let start = usize::from(self.start);
        if id >= start && id < usize::from(self.end) {
            Some(id - start)
        } else {
            None
        }
    }
    //
    // An From/Into would be nicer, but it's dangerously also works for (usize, usize)
    // which could also be interpreted as a range (start, end).
    pub fn from_start_and_length<Src>((start, len): (Src, usize)) -> Self
    where
        Id: From<Src>,
    {
        let start = Id::from(start);
        Self {
            start,
            end: Id::from(usize::from(start) + len),
        }
    }

    // An From/Into would be nicer, but it's dangerously also works for (usize, usize)
    // which could also be interpreted as a range (start, end).
    pub fn from_start_and_end<Src>(start: Src, end: Src) -> Self
    where
        Id: From<Src>,
    {
        Self {
            start: Id::from(start),
            end: Id::from(end),
        }
    }

    pub fn from_single(id: Id) -> Self {
        Self {
            start: id,
            end: Id::from(usize::from(id) + 1),
        }
    }

    pub fn from_slice(ids: &[Id]) -> Option<Self> {
        let mut ids = ids.iter();
        let Some(first) = ids.next() else {
            return Some(Self::empty());
        };
        let start = usize::from(*first);
        let mut end = start;
        for id in ids {
            if usize::from(*id) != end + 1 {
                return None;
            }
            end += 1;
        }
        Some(Self {
            start: *first,
            end: Id::from(end + 1),
        })
    }

    pub fn as_usize(self) -> Range<usize> {
        self.into()
    }
}

impl<Id, T> From<Range<T>> for IdRange<Id>
where
    Id: From<T> + Copy,
{
    fn from(Range { start, end }: Range<T>) -> Self {
        Self {
            start: Id::from(start),
            end: Id::from(end),
        }
    }
}

impl<Id> From<IdRange<Id>> for Range<usize>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    fn from(value: IdRange<Id>) -> Self {
        Range {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

// Hint: Go to the definition of Id instead to find the Walker type.
//       Walk implementations are always close to T.
impl<Ctx, Id: Walk<Ctx> + 'static> Walk<Ctx> for IdRange<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
    Ctx: Copy,
{
    type Walker<'a> = WalkIterator<'a, IdRangeIterator<Id>, Ctx>
    where Ctx: 'a;

    fn walk<'a>(self, ctx: impl Into<Ctx>) -> Self::Walker<'a>
    where
        Self: 'a,
        Ctx: 'a,
    {
        WalkIterator::new(self.into_iter(), ctx.into())
    }
}

impl<Id> IntoIterator for IdRange<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    type Item = Id;
    type IntoIter = IdRangeIterator<Id>;

    fn into_iter(self) -> Self::IntoIter {
        IdRangeIterator(self)
    }
}

#[derive(Clone)]
pub struct IdRangeIterator<Id: Copy>(IdRange<Id>);

impl<Id> Iterator for IdRangeIterator<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.0.is_empty() {
            let id = self.0.start;
            self.0.start = Id::from(usize::from(id) + 1);
            Some(id)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }
}

impl<Id> ExactSizeIterator for IdRangeIterator<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Id> DoubleEndedIterator for IdRangeIterator<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.0.is_empty() {
            self.0.end = Id::from(usize::from(self.0.end) - 1);
            Some(self.0.end)
        } else {
            None
        }
    }
}

impl<Id> std::iter::FusedIterator for IdRangeIterator<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
}
