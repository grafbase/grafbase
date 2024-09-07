use std::cmp::Ordering;

use crate::{Walk, Walker};

pub struct WalkIterator<'g, I, G> {
    iter: I,
    graph: &'g G,
}

impl<'g, I, G> Clone for WalkIterator<'g, I, G>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            graph: self.graph,
        }
    }
}

impl<'g, I, G> Copy for WalkIterator<'g, I, G> where I: Copy {}

impl<'g, I, G> WalkIterator<'g, I, G> {
    pub fn new(iter: I, graph: &'g G) -> Self {
        Self { iter, graph }
    }
}

impl<'g, I, G> Iterator for WalkIterator<'g, I, G>
where
    I: Iterator,
    <I as Iterator>::Item: Walk<G> + 'g,
{
    type Item = Walker<'g, <I as Iterator>::Item, G>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| item.walk(self.graph))
    }
}

impl<'g, I, G> ExactSizeIterator for WalkIterator<'g, I, G>
where
    I: ExactSizeIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'g, I, G> std::iter::FusedIterator for WalkIterator<'g, I, G>
where
    I: std::iter::FusedIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
{
}

impl<'g, I, G> std::iter::DoubleEndedIterator for WalkIterator<'g, I, G>
where
    I: std::iter::DoubleEndedIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| item.walk(self.graph))
    }
}

impl<'g, I, G> std::fmt::Debug for WalkIterator<'g, I, G>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
    I: Clone,
    <Self as Iterator>::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'g, I, G> std::cmp::Ord for WalkIterator<'g, I, G>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
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

impl<'g, I, G> std::cmp::PartialEq for WalkIterator<'g, I, G>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.clone().zip(other.clone()).all(|(left, right)| left == right)
    }
}

impl<'g, I, G> std::cmp::Eq for WalkIterator<'g, I, G>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::Eq,
{
}

impl<'g, I, G> std::cmp::PartialOrd for WalkIterator<'g, I, G>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Walk<G> + 'g,
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
