mod map;

use std::num::NonZero;

pub(in crate::solve) use map::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(in crate::solve) struct DeduplicationId(pub NonZero<u16>); // reserving 0
