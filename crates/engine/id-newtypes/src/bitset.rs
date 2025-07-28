use std::fmt::Binary;

use crate::IdRange;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct BitSet<Id> {
    inner: fixedbitset::FixedBitSet,
    _phantom: std::marker::PhantomData<Id>,
}

impl<Id> Default for BitSet<Id> {
    fn default() -> Self {
        Self {
            inner: fixedbitset::FixedBitSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Id> BitSet<Id>
where
    usize: From<Id>,
    Id: Copy,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(n: usize) -> Self {
        Self {
            inner: fixedbitset::FixedBitSet::with_capacity(n),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn put(&mut self, id: Id) -> bool {
        self.inner.put(usize::from(id))
    }

    pub fn set(&mut self, id: Id, value: bool) {
        self.inner.set(usize::from(id), value)
    }

    pub fn set_range(&mut self, id: IdRange<Id>, value: bool) {
        let start = usize::from(id.start);
        let end = usize::from(id.end);
        self.inner.set_range(start..end, value);
    }

    pub fn push(&mut self, value: bool) {
        self.inner.grow(self.inner.len() + 1);
        self.inner.set(self.inner.len() - 1, value);
    }

    pub fn grow(&mut self, n: usize) {
        self.inner.grow(n)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn zeroes(&self) -> impl Iterator<Item = Id> + '_
    where
        Id: From<usize>,
    {
        self.inner.zeroes().map(|ix| Id::from(ix))
    }

    pub fn ones(&self) -> impl Iterator<Item = Id> + '_
    where
        Id: From<usize>,
    {
        self.inner.ones().map(|ix| Id::from(ix))
    }

    pub fn set_all(&mut self, value: bool) {
        self.inner.set_range(.., value);
    }
}

impl<Id> std::ops::Not for BitSet<Id> {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        self.inner.toggle_range(..);
        self
    }
}

impl<Id> std::ops::BitOr for BitSet<Id> {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.inner |= rhs.inner;
        self
    }
}

impl<Id> std::ops::BitAnd for BitSet<Id> {
    type Output = Self;

    fn bitand(mut self, rhs: Self) -> Self::Output {
        self.inner &= rhs.inner;
        self
    }
}

impl<Id> std::ops::BitXor for BitSet<Id> {
    type Output = Self;

    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self.inner ^= rhs.inner;
        self
    }
}

impl<Id> std::ops::BitAndAssign for BitSet<Id> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.inner &= rhs.inner;
    }
}

impl<Id> std::ops::BitOrAssign for BitSet<Id> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.inner |= rhs.inner;
    }
}

impl<Id> std::ops::BitXorAssign for BitSet<Id> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.inner ^= rhs.inner;
    }
}

impl<Id> std::ops::Index<Id> for BitSet<Id>
where
    usize: From<Id>,
{
    type Output = bool;
    fn index(&self, index: Id) -> &Self::Output {
        &self.inner[usize::from(index)]
    }
}

impl<Id> Binary for BitSet<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitset() {
        let mut bitset = BitSet::<usize>::with_capacity(129);
        for i in 0..129 {
            assert!(!bitset[i]);
        }

        bitset.set(100, true);
        assert!(!bitset[99]);
        assert!(bitset[100]);
        assert!(!bitset[101]);
    }
}
