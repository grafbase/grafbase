use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrappingType {
    NonNull,
    List,
}

/// GraphQL wrappers encoded into a single u32
///
/// Bit 0: Whether the inner type is null
/// Bits 1..5: Number of list wrappers
/// Bits 5..21: List wrappers, where 0 is nullable 1 is non-null
/// The rest: dead bits
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct TypeWrappers(u32);

static INNER_NULLABILITY_MASK: u32 = 1;
static NUM_LISTS_MASK: u32 = 32 - 2;
static NON_NUM_LISTS_MASK: u32 = u32::MAX ^ NUM_LISTS_MASK;

impl TypeWrappers {
    pub fn none() -> Self {
        TypeWrappers(0)
    }

    pub fn wrap_list(&self) -> Self {
        let current_wrappers = self.num_list_wrappers();

        let new_wrappers = current_wrappers + 1;
        assert!(new_wrappers < 16, "list wrapper overflow");

        Self((new_wrappers << 1) | (self.0 & NON_NUM_LISTS_MASK))
    }

    pub fn wrap_non_null(&self) -> Self {
        let index = self.num_list_wrappers();
        if index == 0 {
            return Self(INNER_NULLABILITY_MASK);
        }

        let new = self.0 | (1 << (4 + index));

        TypeWrappers(new)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// iterates oveer the type wrappers from outermost to innermost
    pub fn iter(&self) -> TypeWrappersIter {
        let current_wrappers = self.num_list_wrappers();
        TypeWrappersIter {
            encoded: self.0,
            mask: (1 << (4 + current_wrappers)),
            next: None,
            last: ((INNER_NULLABILITY_MASK & self.0) == INNER_NULLABILITY_MASK).then_some(WrappingType::NonNull),
        }
    }

    pub(crate) fn write_type_string(
        self,
        name: &str,
        mut formatter: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        let wrappers = self.iter().collect::<Vec<_>>();

        for wrapping in &wrappers {
            if let WrappingType::List = wrapping {
                write!(formatter, "[")?;
            }
        }
        write!(formatter, "{name}")?;
        for wrapping in wrappers.iter().rev() {
            match wrapping {
                WrappingType::NonNull => write!(&mut formatter, "!")?,
                WrappingType::List => write!(&mut formatter, "]")?,
            };
        }

        Ok(())
    }

    fn num_list_wrappers(&self) -> u32 {
        (self.0 & NUM_LISTS_MASK) >> 1
    }
}

/// Takes type wrappers from outermost to innermost
impl FromIterator<WrappingType> for TypeWrappers {
    fn from_iter<T: IntoIterator<Item = WrappingType>>(iter: T) -> Self {
        let wrappers = iter.into_iter().collect::<Vec<_>>();

        wrappers
            .into_iter()
            .rev()
            .fold(TypeWrappers::none(), |wrappers, wrapping| match wrapping {
                WrappingType::NonNull => wrappers.wrap_non_null(),
                WrappingType::List => wrappers.wrap_list(),
            })
    }
}

pub struct TypeWrappersIter {
    encoded: u32,
    mask: u32,
    next: Option<WrappingType>,
    last: Option<WrappingType>,
}

impl Iterator for TypeWrappersIter {
    type Item = WrappingType;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            return Some(next);
        }
        if (self.mask & NUM_LISTS_MASK) != 0 {
            if let Some(last) = self.last.take() {
                return Some(last);
            }
            return None;
        }

        // Otherwise we still have list wrappers
        let current_is_non_null = (self.encoded & self.mask) != 0;
        self.mask >>= 1;

        if current_is_non_null {
            self.next = Some(WrappingType::List);
            Some(WrappingType::NonNull)
        } else {
            Some(WrappingType::List)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TypeWrappers, WrappingType};

    #[test]
    fn test_wrappers() {
        assert_eq!(TypeWrappers::none().iter().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            TypeWrappers::none().wrap_non_null().iter().collect::<Vec<_>>(),
            vec![WrappingType::NonNull]
        );

        assert_eq!(
            TypeWrappers::none().wrap_list().iter().collect::<Vec<_>>(),
            vec![WrappingType::List]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .wrap_list()
                .iter()
                .collect::<Vec<_>>(),
            vec![WrappingType::List, WrappingType::NonNull]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .wrap_list()
                .wrap_non_null()
                .iter()
                .collect::<Vec<_>>(),
            vec![WrappingType::NonNull, WrappingType::List, WrappingType::NonNull]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_list()
                .wrap_list()
                .wrap_list()
                .wrap_non_null()
                .iter()
                .collect::<Vec<_>>(),
            vec![
                WrappingType::NonNull,
                WrappingType::List,
                WrappingType::List,
                WrappingType::List,
            ]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .wrap_list()
                .wrap_non_null()
                .wrap_list()
                .iter()
                .collect::<Vec<_>>(),
            vec![
                WrappingType::List,
                WrappingType::NonNull,
                WrappingType::List,
                WrappingType::NonNull
            ]
        );
    }
}
