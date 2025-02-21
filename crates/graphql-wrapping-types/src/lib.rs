mod mutable;

pub use mutable::MutableWrapping;

const LIST_WRAPPER_LENGTH_MASK: u16 = 0b0111_1000_0000_0000;
const LIST_WRAPPER_SHIFT: u32 = LIST_WRAPPER_LENGTH_MASK.trailing_zeros();
const LIST_WRAPPER_MASK: u16 = 0b0000_0111_1111_1111;
const MAX_LIST_WRAPINGS: u32 = LIST_WRAPPER_MASK.trailing_ones();
const INNER_IS_REQUIRED_FLAG: u16 = 0b1000_0000_0000_0000;

/// It's structured as follows:
///
///```text
///       list wrapper length (4 bits)
//        |
///       ↓     ↓ list_wrapping (1 == Required / 0 == Nullable)
///   ┌───┐┌───────────┐
///  0000_0000_0000_0000
///  ↑
///  inner_is_required flag (1 == required)
///```
///
/// The list_wrapping is stored from innermost to outermost and use the start and end
/// as the positions within the list_wrapping bits. Acting like a simplified fixed capacity VecDeque.
/// For simplicity of bit shifts the list wrapping is stored from right to left.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Wrapping(u16);

impl Default for Wrapping {
    fn default() -> Self {
        Self::nullable()
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
    ) -> impl DoubleEndedIterator<Item = ListWrapping> + ExactSizeIterator<Item = ListWrapping> {
        (0..self.get_list_length()).map(move |i| {
            if self.0 & (1 << i) == 0 {
                ListWrapping::NullableList
            } else {
                ListWrapping::RequiredList
            }
        })
    }

    fn get_list_length(&self) -> u8 {
        ((self.0 & LIST_WRAPPER_LENGTH_MASK) >> LIST_WRAPPER_SHIFT) as u8
    }

    fn set_list_length(&mut self, len: u8) {
        assert!((len as u32) < MAX_LIST_WRAPINGS, "list wrapper overflow");
        self.0 = (self.0 & !LIST_WRAPPER_LENGTH_MASK) | ((len as u16) << LIST_WRAPPER_SHIFT);
    }

    pub fn to_mutable(self) -> MutableWrapping {
        self.into()
    }

    pub fn nullable() -> Self {
        Wrapping(0)
    }

    pub fn required() -> Self {
        Wrapping(INNER_IS_REQUIRED_FLAG)
    }

    #[must_use]
    pub fn wrap_list(mut self) -> Self {
        let len = self.get_list_length();
        self.set_list_length(len + 1);
        self.0 &= !(1 << len);
        self
    }

    #[must_use]
    pub fn wrap_list_non_null(mut self) -> Self {
        let len = self.get_list_length();
        self.set_list_length(len + 1);
        self.0 |= 1 << len;
        self
    }

    #[must_use]
    pub fn wrap_non_null(mut self) -> Self {
        let len = self.get_list_length();
        if len == 0 {
            self.0 |= INNER_IS_REQUIRED_FLAG;
        } else {
            self.0 |= 1 << (len - 1);
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListWrapping {
    RequiredList,
    NullableList,
}

impl Wrapping {
    pub fn new(required: bool) -> Self {
        if required { Self::required() } else { Self::nullable() }
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

    pub fn write_type_string(self, name: &str, mut formatter: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        for _ in 0..self.list_wrappings().len() {
            write!(formatter, "[")?;
        }

        write!(formatter, "{name}")?;

        if self.inner_is_required() {
            write!(formatter, "!")?;
        }

        for wrapping in self.list_wrappings() {
            match wrapping {
                ListWrapping::RequiredList => write!(&mut formatter, "]!")?,
                ListWrapping::NullableList => write!(&mut formatter, "]")?,
            };
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapping() {
        let wrapping = Wrapping::required();
        assert!(wrapping.inner_is_required());
        assert!(wrapping.is_required() && !wrapping.is_nullable());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        let mut wrapping = Wrapping::nullable();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_nullable() && !wrapping.is_required());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        wrapping = wrapping.wrap_list();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_nullable() && !wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::NullableList]
        );

        wrapping = wrapping.wrap_list_non_null();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_required() && !wrapping.is_nullable());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::NullableList, ListWrapping::RequiredList]
        );

        wrapping = wrapping.wrap_list();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_nullable() && !wrapping.is_required());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![
                ListWrapping::NullableList,
                ListWrapping::RequiredList,
                ListWrapping::NullableList
            ]
        );
    }

    #[test]
    fn test_mutable_wrapping() {
        let mut wrapping = Wrapping::default()
            .wrap_non_null()
            .wrap_list()
            .wrap_list_non_null()
            .wrap_list()
            .to_mutable();

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::NullableList));

        assert!(wrapping.is_required());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::RequiredList));

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::NullableList));

        assert!(wrapping.is_required());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), None);

        let mut wrapping = Wrapping::default().wrap_list().wrap_list_non_null().to_mutable();

        assert!(wrapping.is_required());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::RequiredList));

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::NullableList));

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), None);
    }

    #[test]
    fn test_wrapping_order() {
        let wrapping = Wrapping::default()
            .wrap_non_null()
            .wrap_list()
            .wrap_list()
            .wrap_list_non_null()
            .wrap_list();
        assert!(wrapping.inner_is_required());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![
                ListWrapping::NullableList,
                ListWrapping::NullableList,
                ListWrapping::RequiredList,
                ListWrapping::NullableList
            ]
        );

        let wrapping = Wrapping::required()
            .wrap_list()
            .wrap_list()
            .wrap_list_non_null()
            .wrap_list()
            .wrap_non_null();
        assert!(wrapping.inner_is_required());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![
                ListWrapping::NullableList,
                ListWrapping::NullableList,
                ListWrapping::RequiredList,
                ListWrapping::RequiredList
            ]
        );

        let mut wrapping = Wrapping::required()
            .wrap_list()
            .wrap_list()
            .wrap_list_non_null()
            .wrap_list()
            .to_mutable();
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::NullableList));
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::RequiredList));
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::NullableList));
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::NullableList));
    }

    #[test]
    fn back_and_forth() {
        let original = Wrapping::required().wrap_list_non_null();
        let mut wrapping = original.to_mutable();
        let list_wrapping = wrapping.pop_outermost_list_wrapping().unwrap();
        wrapping.push_outermost_list_wrapping(list_wrapping);
        assert_eq!(Wrapping::from(wrapping), original);

        let original = Wrapping::nullable().wrap_list();
        let mut wrapping = original.to_mutable();
        let list_wrapping = wrapping.pop_outermost_list_wrapping().unwrap();
        wrapping.push_outermost_list_wrapping(list_wrapping);
        assert_eq!(Wrapping::from(wrapping), original);
    }
}
