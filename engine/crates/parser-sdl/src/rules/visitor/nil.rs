use super::{Visitor, VisitorCons};

/// Empty Value
pub struct VisitorNil;

impl VisitorNil {
    pub(crate) const fn with<V>(self, visitor: V) -> VisitorCons<V, Self> {
        VisitorCons(visitor, self)
    }
}

impl<'a> Visitor<'a> for VisitorNil {}
