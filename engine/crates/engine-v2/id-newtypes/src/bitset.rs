use bitvec::{bitvec, vec::BitVec};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BitSet<Id> {
    inner: BitVec,
    _phantom: std::marker::PhantomData<Id>,
}

impl<Id> Default for BitSet<Id> {
    fn default() -> Self {
        Self {
            inner: BitVec::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Id> BitSet<Id>
where
    usize: From<Id>,
{
    pub fn init_with(value: bool, n: usize) -> Self {
        Self {
            inner: bitvec![value as usize; n],
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set(&mut self, id: Id, value: bool) {
        self.inner.set(usize::from(id), value)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitset() {
        let bitset = BitSet::<usize>::init_with(true, 129);
        for i in 0..129 {
            assert!(bitset[i]);
        }

        let mut bitset = BitSet::<usize>::init_with(false, 129);
        for i in 0..129 {
            assert!(!bitset[i]);
        }

        bitset.set(100, true);
        assert!(!bitset[99]);
        assert!(bitset[100]);
        assert!(!bitset[101]);
    }
}
