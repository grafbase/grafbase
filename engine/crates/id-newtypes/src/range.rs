use std::ops::Range;

// Not necessary anymore when Rust stabilize std::iter::Step
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IdRange<Id: Copy> {
    pub start: Id,
    pub end: Id,
}

impl<Id: Copy + From<usize>> Default for IdRange<Id> {
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

    pub fn from_single(id: Id) -> Self {
        Self {
            start: id,
            end: Id::from(usize::from(id) + 1),
        }
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

impl<Id> Iterator for IdRange<Id>
where
    Id: Copy + From<usize>,
    usize: From<Id>,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if !IdRange::<Id>::is_empty(self) {
            let id = self.start;
            self.start = Id::from(usize::from(id) + 1);
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

impl<Id> ExactSizeIterator for IdRange<Id>
where
    Id: Copy + From<usize>,
    usize: From<Id>,
{
    fn len(&self) -> usize {
        IdRange::len(self)
    }
}
