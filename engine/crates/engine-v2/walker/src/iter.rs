use std::{cmp::Ordering, marker::PhantomData};

use crate::{Walk, Walker};

#[derive(Clone, Copy)]
pub struct WalkIterator<'a, I, Ctx> {
    iter: I,
    ctx: Ctx,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, I, Ctx> WalkIterator<'a, I, Ctx> {
    pub fn new(iter: I, ctx: Ctx) -> Self
    where
        I: 'a,
        Ctx: Copy + 'a,
    {
        Self {
            iter,
            ctx,
            _phantom: PhantomData,
        }
    }
}

impl<'a, I, Ctx> Iterator for WalkIterator<'a, I, Ctx>
where
    I: Iterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: 'a,
    Ctx: Copy + 'a,
{
    type Item = Walker<'a, <I as Iterator>::Item, Ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| item.walk(self.ctx))
    }
}

impl<'a, I, Ctx> ExactSizeIterator for WalkIterator<'a, I, Ctx>
where
    I: ExactSizeIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: 'a,
    Ctx: Copy + 'a,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, I, Ctx> std::iter::FusedIterator for WalkIterator<'a, I, Ctx>
where
    I: std::iter::FusedIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: 'a,
    Ctx: Copy + 'a,
{
}

impl<'a, I, Ctx> std::iter::DoubleEndedIterator for WalkIterator<'a, I, Ctx>
where
    I: std::iter::DoubleEndedIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: 'a,
    Ctx: Copy + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| item.walk(self.ctx))
    }
}

impl<'a, I, Ctx> std::fmt::Debug for WalkIterator<'a, I, Ctx>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: Clone + 'a,
    Ctx: Copy + 'a,
    <Self as Iterator>::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

// `@requires` fields can have arguments and we intern each individual field to allow quick
// comparison of fields when merging field sets. So we rely on the Ord here to compare argument
// lists.
impl<'a, I, Ctx> std::cmp::Ord for WalkIterator<'a, I, Ctx>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: Clone + 'a,
    Ctx: Copy + 'a,
    <Self as Iterator>::Item: std::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.len().cmp(&other.len()).then_with(|| {
            for (left, right) in self.clone().zip(other.clone()) {
                match left.cmp(&right) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            Ordering::Equal
        })
    }
}

impl<'a, I, Ctx> std::cmp::PartialEq for WalkIterator<'a, I, Ctx>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: Clone + 'a,
    Ctx: Copy + 'a,
    <Self as Iterator>::Item: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.clone().zip(other.clone()).all(|(left, right)| left == right)
    }
}

impl<'a, I, Ctx> std::cmp::Eq for WalkIterator<'a, I, Ctx>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: Clone + 'a,
    Ctx: Copy + 'a,
    <Self as Iterator>::Item: std::cmp::Eq,
{
}

impl<'a, I, Ctx> std::cmp::PartialOrd for WalkIterator<'a, I, Ctx>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<Ctx> + 'a,
    I: Clone + 'a,
    Ctx: Copy + 'a,
    <Self as Iterator>::Item: std::cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.len().partial_cmp(&other.len()) {
            Some(Ordering::Equal) => (),
            other => return other,
        };

        for (left, right) in self.clone().zip(other.clone()) {
            match left.partial_cmp(&right) {
                Some(Ordering::Equal) => continue,
                other => return other,
            }
        }

        Some(Ordering::Equal)
    }
}
