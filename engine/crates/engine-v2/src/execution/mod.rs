#[derive(Debug)]
pub struct Strings(lasso::Rodeo<StrId>);

impl Strings {
    pub fn new() -> Self {
        Self(lasso::Rodeo::new())
    }

    pub fn get_or_intern(&mut self, s: &str) -> StrId {
        self.0.get_or_intern(s)
    }
}

impl std::ops::Index<StrId> for Strings {
    type Output = str;

    fn index(&self, index: StrId) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrId(u32);

// Reserving the 4 upper bits for flags which still leaves 268 millions ids.
const ID_MASK: usize = 0x0F_FF_FF_FF;

unsafe impl lasso::Key for StrId {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn try_from_usize(int: usize) -> Option<Self> {
        if int < ID_MASK {
            Some(Self(int as u32))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use lasso::Key;

    use super::*;

    #[test]
    fn field_name_value_in_range() {
        let key = StrId::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = StrId::try_from_usize(ID_MASK - 1).unwrap();
        assert_eq!(key.into_usize(), ID_MASK - 1);
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = StrId::try_from_usize(ID_MASK);
        assert!(key.is_none());

        let key = StrId::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
