use std::num::NonZeroU16;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, derive_more::Display)]
pub struct PlanId(NonZeroU16);

impl From<usize> for PlanId {
    fn from(value: usize) -> Self {
        PlanId(NonZeroU16::new((value + 1).try_into().expect("Too many plans.")).unwrap())
    }
}

impl From<PlanId> for usize {
    fn from(value: PlanId) -> Self {
        (value.0.get() - 1) as usize
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, derive_more::Display)]
pub struct PlanBoundaryId(NonZeroU16);

impl From<usize> for PlanBoundaryId {
    fn from(value: usize) -> Self {
        PlanBoundaryId(NonZeroU16::new((value + 1).try_into().expect("Too many plan boundaries.")).unwrap())
    }
}

impl From<PlanBoundaryId> for usize {
    fn from(value: PlanBoundaryId) -> Self {
        (value.0.get() - 1) as usize
    }
}
