//! Wrapping is compacted into a u32 to be Copy. It's copied at various places to keep track of
//! current wrapping. It's functionally equivalent to:
//!
//! ```ignore
//! struct Wrapping {
//!   inner_is_required: bool,
//!   /// Innermost to outermost.
//!   list_wrappings: VecDeque<ListWrapping>
//! }
//! ```
//!
//! Since ListWrapping has only two cases and we won't encounter absurd levels of wrapping, we
//! can bitpack it. The current structure supports up to 21 list_wrappings.
//!
//! It's structured as follows:
//!
//!```text
//!       start (5 bits)
//!       ↓               ↓ list_wrapping (1 == Required / 0 == Nullable)
//!   ┌────┐      ┌────────────────────────┐
//!  0000_0000_0000_0000_0000_0000_0000_0000
//!         └────┘
//!            ↑ end (5 bits)
//!  ↑
//!  inner_is_required flag (1 == required)
//!```
//!
//! The list_wrapping is stored from innermost to outermost and use the start and end
//! as the positions within the list_wrapping bits. Acting like a simplified fixed capacity VecDeque.
//! For simplicity of bit shifts the list wrapping is stored from right to left.
const START_MASK: u32 = 0b0111_1100_0000_0000_0000_0000_0000_0000;
const START_SHIFT: u32 = START_MASK.trailing_zeros();
const END_MASK: u32 = 0b0000_0011_1110_0000_0000_0000_0000_0000;
const END_SHIFT: u32 = END_MASK.trailing_zeros();
const LIST_WRAPPINGS_MASK: u32 = 0b0000_0000_0001_1111_1111_1111_1111_1111;
const MAX_LIST_WRAPINGS: u32 = LIST_WRAPPINGS_MASK.trailing_ones();
const INNER_IS_REQUIRED_FLAG: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Wrapping(u32);

impl Default for Wrapping {
    fn default() -> Self {
        Self::nullable()
    }
}

impl From<Wrapping> for u32 {
    fn from(value: Wrapping) -> Self {
        value.0
    }
}

impl From<u32> for Wrapping {
    fn from(value: u32) -> Self {
        Wrapping(value)
    }
}

impl std::fmt::Debug for Wrapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wrapping")
            .field("inner_is_required", &self.inner_is_required())
            .field("list_wrappings", &self.list_wrappings().collect::<Vec<_>>())
            .finish()
    }
}

impl Wrapping {
    /// Is the innermost type required?
    ///
    /// Examples:
    ///
    /// - `String` => false
    /// - `String!` => true
    /// - `[String!]` => true
    /// - `[String]!` => false
    pub fn inner_is_required(self) -> bool {
        self.0 & INNER_IS_REQUIRED_FLAG != 0
    }

    /// Innermost to outermost.
    pub fn list_wrappings(
        self,
    ) -> impl DoubleEndedIterator<Item = ListWrapping> + Copy + ExactSizeIterator<Item = ListWrapping> {
        self
    }

    pub fn is_nullable(self) -> bool {
        !self.is_required()
    }

    pub fn is_required(self) -> bool {
        self.list_wrappings()
            .next_back()
            .map(|lw| matches!(lw, ListWrapping::RequiredList))
            .unwrap_or(self.inner_is_required())
    }

    pub fn is_list(self) -> bool {
        self.list_wrappings().next().is_some()
    }

    fn start(self) -> u32 {
        (self.0 & START_MASK) >> START_SHIFT
    }

    fn inc_start(&mut self) {
        let start = self.start() + 1;
        self.0 = (self.0 & !START_MASK) | (start << START_SHIFT);
    }

    fn end(self) -> u32 {
        (self.0 & END_MASK) >> END_SHIFT
    }
    fn inc_end(&mut self) {
        let end = self.end() + 1;
        assert!(end < MAX_LIST_WRAPINGS + 1, "Too many list wrappings");
        self.0 = (self.0 & !END_MASK) | (end << END_SHIFT);
    }

    fn dec_end(&mut self) {
        let end = self.end() - 1;
        self.0 = (self.0 & !END_MASK) | (end << END_SHIFT);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListWrapping {
    RequiredList,
    NullableList,
}

impl Iterator for Wrapping {
    type Item = ListWrapping;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.start();
        if start == self.end() {
            return None;
        }
        self.inc_start();
        let bit_mask = 1 << start;
        if self.0 & bit_mask != 0 {
            Some(ListWrapping::RequiredList)
        } else {
            Some(ListWrapping::NullableList)
        }
    }
}

impl ExactSizeIterator for Wrapping {
    fn len(&self) -> usize {
        (self.end() - self.start()) as usize
    }
}

impl DoubleEndedIterator for Wrapping {
    fn next_back(&mut self) -> Option<Self::Item> {
        let end = self.end();
        if end == self.start() {
            return None;
        }
        self.dec_end();
        // end is exclusive
        let bit_mask = 1 << (end - 1);
        if self.0 & bit_mask != 0 {
            Some(ListWrapping::RequiredList)
        } else {
            Some(ListWrapping::NullableList)
        }
    }
}

impl Wrapping {
    pub fn new(required: bool) -> Self {
        if required {
            Self::required()
        } else {
            Self::nullable()
        }
    }

    pub fn nullable() -> Self {
        Wrapping(0)
    }

    pub fn required() -> Self {
        Wrapping(INNER_IS_REQUIRED_FLAG)
    }

    #[must_use]
    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        match list_wrapping {
            ListWrapping::RequiredList => self.wrapped_by_required_list(),
            ListWrapping::NullableList => self.wrapped_by_nullable_list(),
        }
    }

    #[must_use]
    pub fn wrapped_by_nullable_list(mut self) -> Self {
        let end = self.end();
        self.inc_end();
        let bit_mask = 1 << end;
        self.0 &= !bit_mask;
        self
    }

    #[must_use]
    pub fn wrapped_by_required_list(mut self) -> Self {
        let end = self.end();
        self.inc_end();
        let bit_mask = 1 << end;
        self.0 |= bit_mask;
        self
    }

    /// Outermost wrapping
    pub fn pop_list_wrapping(&mut self) -> Option<ListWrapping> {
        self.next_back()
    }

    pub fn write_type_string(self, name: &str, mut formatter: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        for _ in 0..self.len() {
            write!(formatter, "[")?;
        }

        write!(formatter, "{name}")?;

        if self.inner_is_required() {
            write!(formatter, "!")?;
        }

        for wrapping in self {
            match wrapping {
                ListWrapping::RequiredList => write!(&mut formatter, "]!")?,
                ListWrapping::NullableList => write!(&mut formatter, "]")?,
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapping() {
        let wrapping = Wrapping::required();
        assert!(wrapping.inner_is_required());
        assert!(wrapping.is_required());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        let mut wrapping = Wrapping::nullable();
        assert!(!wrapping.inner_is_required());
        assert!(!wrapping.is_required());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        wrapping = wrapping.wrapped_by_nullable_list();
        assert!(!wrapping.inner_is_required());
        assert!(!wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::NullableList]
        );

        wrapping = wrapping.wrapped_by_required_list();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::NullableList, ListWrapping::RequiredList]
        );

        wrapping = wrapping.wrapped_by_nullable_list();
        assert!(!wrapping.inner_is_required());
        assert!(!wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![
                ListWrapping::NullableList,
                ListWrapping::RequiredList,
                ListWrapping::NullableList
            ]
        );

        assert_eq!(wrapping.pop_list_wrapping(), Some(ListWrapping::NullableList));
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::NullableList, ListWrapping::RequiredList]
        );

        assert_eq!(wrapping.pop_list_wrapping(), Some(ListWrapping::RequiredList));
        assert!(!wrapping.inner_is_required());
        assert!(!wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::NullableList]
        );

        assert_eq!(wrapping.pop_list_wrapping(), Some(ListWrapping::NullableList));
        assert!(!wrapping.inner_is_required());
        assert!(!wrapping.is_required());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        assert_eq!(wrapping.pop_list_wrapping(), None);
    }
}
