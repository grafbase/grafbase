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
///      list wrapper length (4 bits)
//       |
///      ↓     ↓ list_wrapping (1 == Required / 0 == Nullable)
///   ┌───┐┌───────────┐
///  0000_0000_0000_0000
///  ↑
///  inner_is_required flag (1 == required)
///```
///
/// The list_wrapping is stored from innermost to outermost and use the start and end
/// as the positions within the list_wrapping bits. Acting like a simplified fixed capacity VecDeque.
/// For simplicity of bit shifts the list wrapping is stored from right to left.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Wrapping(u16);

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
                ListWrapping::List
            } else {
                ListWrapping::ListNonNull
            }
        })
    }

    /// From innermost to outermost.
    pub fn iter(self) -> impl Iterator<Item = WrappingType> {
        [self.inner_is_required().then_some(WrappingType::NonNull)]
            .into_iter()
            .chain(self.list_wrappings().flat_map(|lw| match lw {
                ListWrapping::List => [Some(WrappingType::List), None],
                ListWrapping::ListNonNull => [Some(WrappingType::List), Some(WrappingType::NonNull)],
            }))
            .flatten()
    }

    const fn get_list_length(&self) -> u8 {
        ((self.0 & LIST_WRAPPER_LENGTH_MASK) >> LIST_WRAPPER_SHIFT) as u8
    }

    const fn set_list_length(&mut self, len: u8) {
        assert!((len as u32) < MAX_LIST_WRAPINGS, "list wrapper overflow");
        self.0 = (self.0 & INNER_IS_REQUIRED_FLAG) | ((len as u16) << LIST_WRAPPER_SHIFT) | (self.0 & ((1 << len) - 1));
    }

    pub fn to_mutable(self) -> MutableWrapping {
        self.into()
    }

    #[must_use]
    pub const fn list(mut self) -> Self {
        let len = self.get_list_length();
        self.set_list_length(len + 1);
        self.0 &= !(1 << len);
        self
    }

    #[must_use]
    pub const fn list_non_null(mut self) -> Self {
        let len = self.get_list_length();
        self.set_list_length(len + 1);
        self.0 |= 1 << len;
        self
    }

    #[must_use]
    pub const fn non_null(mut self) -> Self {
        let len = self.get_list_length();
        if len == 0 {
            self.0 |= INNER_IS_REQUIRED_FLAG;
        } else {
            self.0 |= 1 << (len - 1);
        }
        self
    }

    /// Whether a type wrapped with Self could receive a type wrapping with other.
    pub fn is_equal_or_more_lenient_than(self, other: Wrapping) -> bool {
        if self.inner_is_required() && !other.inner_is_required() {
            return false;
        }
        if self.get_list_length() != other.get_list_length() {
            return false;
        }
        for (s, o) in self.list_wrappings().zip(other.list_wrappings()) {
            if s == ListWrapping::ListNonNull && o == ListWrapping::List {
                return false;
            }
        }
        true
    }

    pub fn without_list(self) -> Option<Wrapping> {
        let mut wrapping = self.to_mutable();
        wrapping.pop_outermost_list_wrapping().map(|_| wrapping.into())
    }

    pub fn without_non_null(mut self) -> Wrapping {
        if self.is_nullable() {
            self
        } else if self.is_list() {
            let mut wrapping = self.to_mutable();
            wrapping.pop_outermost_list_wrapping();
            wrapping.push_outermost_list_wrapping(ListWrapping::List);
            wrapping.into()
        } else {
            self.0 &= !INNER_IS_REQUIRED_FLAG;
            self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrappingType {
    NonNull,
    List,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListWrapping {
    ListNonNull,
    List,
}

impl Wrapping {
    pub fn is_nullable(self) -> bool {
        !self.is_non_null()
    }

    pub fn is_non_null(self) -> bool {
        self.list_wrappings()
            .next_back()
            .map(|lw| matches!(lw, ListWrapping::ListNonNull))
            .unwrap_or(self.inner_is_required())
    }

    pub fn is_list(self) -> bool {
        self.list_wrappings().next().is_some()
    }

    pub fn type_display(self, name: &str) -> impl std::fmt::Display {
        WrappingDisplay { name, wrapping: self }
    }
}

struct WrappingDisplay<'a> {
    name: &'a str,
    wrapping: Wrapping,
}

impl std::fmt::Display for WrappingDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.wrapping.get_list_length() {
            write!(f, "[")?;
        }

        write!(f, "{}", self.name)?;

        if self.wrapping.inner_is_required() {
            write!(f, "!")?;
        }

        for wrapping in self.wrapping.list_wrappings() {
            match wrapping {
                ListWrapping::ListNonNull => write!(f, "]!")?,
                ListWrapping::List => write!(f, "]")?,
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
            .field("binary", &format!("{:016b}", self.0))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapping() {
        let wrapping = Wrapping::default().non_null();
        assert!(wrapping.inner_is_required());
        assert!(wrapping.is_non_null() && !wrapping.is_nullable());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        let mut wrapping = Wrapping::default();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_nullable() && !wrapping.is_non_null());
        assert!(!wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![]);

        wrapping = wrapping.list();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_nullable() && !wrapping.is_non_null());
        assert!(wrapping.is_list());
        assert_eq!(wrapping.list_wrappings().collect::<Vec<_>>(), vec![ListWrapping::List]);

        wrapping = wrapping.list_non_null();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_non_null() && !wrapping.is_nullable());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::List, ListWrapping::ListNonNull]
        );

        wrapping = wrapping.list();
        assert!(!wrapping.inner_is_required());
        assert!(wrapping.is_nullable() && !wrapping.is_non_null());
        assert!(wrapping.is_list());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![ListWrapping::List, ListWrapping::ListNonNull, ListWrapping::List]
        );
    }

    #[test]
    fn test_mutable_wrapping() {
        let mut wrapping = Wrapping::default()
            .non_null()
            .list()
            .list_non_null()
            .list()
            .to_mutable();

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::List));

        assert!(wrapping.is_required());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::ListNonNull));

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::List));

        assert!(wrapping.is_required());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), None);

        let mut wrapping = Wrapping::default().list().list_non_null().to_mutable();

        assert!(wrapping.is_required());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::ListNonNull));

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::List));

        assert!(wrapping.is_nullable());
        assert_eq!(wrapping.pop_outermost_list_wrapping(), None);
    }

    #[test]
    fn test_wrapping_order() {
        let wrapping = Wrapping::default().non_null().list().list().list_non_null().list();
        assert!(wrapping.inner_is_required());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![
                ListWrapping::List,
                ListWrapping::List,
                ListWrapping::ListNonNull,
                ListWrapping::List
            ]
        );

        let wrapping = Wrapping::default()
            .non_null()
            .list()
            .list()
            .list_non_null()
            .list()
            .non_null();
        assert!(wrapping.inner_is_required());
        assert_eq!(
            wrapping.list_wrappings().collect::<Vec<_>>(),
            vec![
                ListWrapping::List,
                ListWrapping::List,
                ListWrapping::ListNonNull,
                ListWrapping::ListNonNull
            ]
        );

        let mut wrapping = Wrapping::default()
            .non_null()
            .list()
            .list()
            .list_non_null()
            .list()
            .to_mutable();
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::List));
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::ListNonNull));
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::List));
        assert_eq!(wrapping.pop_outermost_list_wrapping(), Some(ListWrapping::List));
    }

    #[test]
    fn back_and_forth() {
        let original = Wrapping::default().non_null().list_non_null();
        let mut wrapping = original.to_mutable();
        let list_wrapping = wrapping.pop_outermost_list_wrapping().unwrap();
        assert_eq!(Wrapping::from(wrapping.clone()), Wrapping::default().non_null());

        wrapping.push_outermost_list_wrapping(list_wrapping);
        assert_eq!(Wrapping::from(wrapping), original);

        let original = Wrapping::default().list();
        let mut wrapping = original.to_mutable();
        let list_wrapping = wrapping.pop_outermost_list_wrapping().unwrap();
        assert_eq!(Wrapping::from(wrapping.clone()), Wrapping::default());

        wrapping.push_outermost_list_wrapping(list_wrapping);
        assert_eq!(Wrapping::from(wrapping), original);
    }

    #[test]
    fn test_is_equal_or_more_lenient_than() {
        let non_null_list = Wrapping::default().non_null().list();
        let non_null_list_non_null = Wrapping::default().non_null().list_non_null();
        assert!(non_null_list.is_equal_or_more_lenient_than(non_null_list_non_null));
        assert!(!non_null_list_non_null.is_equal_or_more_lenient_than(non_null_list));

        let list = Wrapping::default().list();
        assert!(list.is_equal_or_more_lenient_than(non_null_list));
        assert!(!non_null_list.is_equal_or_more_lenient_than(list));

        let list_non_null = Wrapping::default().list_non_null();
        assert!(!non_null_list.is_equal_or_more_lenient_than(list_non_null));
        assert!(!list_non_null.is_equal_or_more_lenient_than(non_null_list));
    }
}
