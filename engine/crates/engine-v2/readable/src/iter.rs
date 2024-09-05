use std::cmp::Ordering;

use crate::{Readable, Reader};

pub struct ReadableIterator<'w, I, W> {
    iter: I,
    world: &'w W,
}

impl<'w, I, W> Clone for ReadableIterator<'w, I, W>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            world: self.world,
        }
    }
}

impl<'w, I, W> Copy for ReadableIterator<'w, I, W> where I: Copy {}

impl<'w, I, W> ReadableIterator<'w, I, W> {
    pub fn new(iter: I, world: &'w W) -> Self {
        Self { iter, world }
    }
}

impl<'w, I, W> Iterator for ReadableIterator<'w, I, W>
where
    I: Iterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    type Item = Reader<'w, <I as Iterator>::Item, W>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| item.read(self.world))
    }
}

impl<'w, I, W> ExactSizeIterator for ReadableIterator<'w, I, W>
where
    I: ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'w, I, W> std::iter::FusedIterator for ReadableIterator<'w, I, W>
where
    I: std::iter::FusedIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
}

impl<'w, I, W> std::iter::DoubleEndedIterator for ReadableIterator<'w, I, W>
where
    I: std::iter::DoubleEndedIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| item.read(self.world))
    }
}

impl<'w, I, W> std::fmt::Debug for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'w, I, W> std::cmp::Ord for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
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

impl<'w, I, W> std::cmp::PartialEq for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.clone().zip(other.clone()).all(|(left, right)| left == right)
    }
}

impl<'w, I, W> std::cmp::Eq for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::Eq,
{
}

impl<'w, I, W> std::cmp::PartialOrd for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
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
