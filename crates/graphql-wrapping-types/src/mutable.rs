use crate::{ListWrapping, Wrapping, INNER_IS_REQUIRED_SHIFT, LIST_WRAPPER_MASK, LIST_WRAPPER_SHIFT};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct MutableWrapping {
    pub(super) start: u8,
    pub(super) inner: Wrapping,
}

impl MutableWrapping {
    pub fn is_nullable(&self) -> bool {
        self.inner.is_nullable()
    }

    pub fn is_required(&self) -> bool {
        self.inner.is_required()
    }

    pub fn pop_outermost_list_wrapping(&mut self) -> Option<ListWrapping> {
        let end = self.inner.get_list_length();
        if self.start == end {
            return None;
        }
        self.inner.set_list_length(end - 1);
        // end is exclusive
        let bit_mask = 1u16 << (end - 1);
        if self.inner.0 & bit_mask != 0 {
            Some(ListWrapping::RequiredList)
        } else {
            Some(ListWrapping::NullableList)
        }
    }

    pub fn push_outermost_list_wrapping(&mut self, list_wrapping: ListWrapping) {
        self.inner = match list_wrapping {
            ListWrapping::RequiredList => self.inner.wrap_list_non_null(),
            ListWrapping::NullableList => self.inner.wrap_list(),
        };
    }

    pub fn pop_innermost_list_wrapping(&mut self) -> Option<ListWrapping> {
        if self.start == self.inner.get_list_length() {
            return None;
        }
        self.start += 1;
        let bit_mask = 1u16 << (self.start - 1);
        if self.inner.0 & bit_mask != 0 {
            Some(ListWrapping::RequiredList)
        } else {
            Some(ListWrapping::NullableList)
        }
    }
}

impl From<MutableWrapping> for Wrapping {
    fn from(wrapping: MutableWrapping) -> Self {
        let MutableWrapping { start, inner } = wrapping;
        let len = inner.get_list_length() - start;
        let inner_is_required = inner.inner_is_required();
        let list_wrappers_bits = (inner.0 & LIST_WRAPPER_MASK) >> start;

        Wrapping(
            list_wrappers_bits
                | ((inner_is_required as u16) << INNER_IS_REQUIRED_SHIFT)
                | ((len as u16) << LIST_WRAPPER_SHIFT),
        )
    }
}

impl From<Wrapping> for MutableWrapping {
    fn from(inner: Wrapping) -> Self {
        let start = 0;
        Self { start, inner }
    }
}

impl Iterator for MutableWrapping {
    type Item = ListWrapping;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_innermost_list_wrapping()
    }
}

impl ExactSizeIterator for MutableWrapping {
    fn len(&self) -> usize {
        (self.inner.get_list_length() - self.start) as usize
    }
}

impl DoubleEndedIterator for MutableWrapping {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pop_outermost_list_wrapping()
    }
}
