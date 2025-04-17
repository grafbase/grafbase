use crate::{ListWrapping, Wrapping};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct MutableWrapping {
    pub(super) inner: Wrapping,
}

impl MutableWrapping {
    pub fn is_nullable(&self) -> bool {
        self.inner.is_nullable()
    }

    pub fn is_required(&self) -> bool {
        self.inner.is_required()
    }

    pub fn write_type_string(&self, name: &str, formatter: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        self.inner.write_type_string(name, formatter)
    }

    pub fn pop_outermost_list_wrapping(&mut self) -> Option<ListWrapping> {
        let end = self.inner.get_list_length();
        if end == 0 {
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
            ListWrapping::RequiredList => self.inner.list_non_null(),
            ListWrapping::NullableList => self.inner.list(),
        };
    }
}

impl From<MutableWrapping> for Wrapping {
    fn from(wrapping: MutableWrapping) -> Self {
        wrapping.inner
    }
}

impl From<Wrapping> for MutableWrapping {
    fn from(inner: Wrapping) -> Self {
        Self { inner }
    }
}
