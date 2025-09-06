mod map;

use std::num::NonZero;

pub(in crate::solve) use map::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(in crate::solve) struct DeduplicationId(pub NonZero<u16>); // reserving 0

#[allow(unused)]
pub(in crate::solve) struct SerializedSolutionGraph(Vec<u16>);

#[allow(unused)]
impl SerializedSolutionGraph {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, id: Option<DeduplicationId>) {
        let mut id: u16 = zerocopy::transmute!(id.map(|id| id.0));
        self.0.push(id);
    }
}
