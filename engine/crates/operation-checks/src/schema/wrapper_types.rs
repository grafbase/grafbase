#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum WrapperType {
    List,
    RequiredList,
    Required,
}

/// The [wrapping types](http://spec.graphql.org/October2021/#sec-Wrapping-Types) for a given
/// instance of a type.
///
/// Implementation: this is a 64 bit integer. Layout is the following, from highest to lowest bits:
///
/// - 6 bits containing an integer representing the number of list wrapping types.
/// - 1 bit representing whether the innermost type is required.
/// - 57 bits representing the list wrappers. The lowest bit is the outermost list wrapping type.
///
/// So we can represent up to 57 levels of list nesting. This should be enough.
#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct WrapperTypes(u64);

impl std::fmt::Debug for WrapperTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter_wrappers()).finish()
    }
}

impl WrapperTypes {
    /// The number of bits taken by the list wrapping types.
    const LIST_BITS_COUNT: u64 = 57;

    /// The offset of the required bit (for shifts).
    const REQUIRED_BIT_OFFSET: u64 = Self::LIST_BITS_COUNT;
    /// Mask for the required bit.
    const REQUIRED_BIT_MASK: u64 = 1 << Self::REQUIRED_BIT_OFFSET;

    /// The offset of the list count integer (for shifts).
    const LIST_COUNT_BITS_OFFSET: u64 = Self::REQUIRED_BIT_OFFSET + 1;
    /// The mask for the list count integer.
    const LIST_COUNT_BITS_MASK: u64 = u64::MAX << Self::LIST_COUNT_BITS_OFFSET;

    fn lists_count(&self) -> u64 {
        (self.0 & Self::LIST_COUNT_BITS_MASK) >> Self::LIST_COUNT_BITS_OFFSET
    }

    pub fn is_required(&self) -> bool {
        self.0 & Self::REQUIRED_BIT_MASK != 0
    }

    /// Iterate wrapper types from outermost to innermost.
    pub(crate) fn iter_wrappers(&self) -> impl Iterator<Item = WrapperType> + '_ {
        (0..self.lists_count())
            .map(move |i| match (self.0 >> i) & 1 {
                0 => WrapperType::List,
                1 => WrapperType::RequiredList,
                _ => unreachable!(),
            })
            .chain(Some(WrapperType::Required).filter(|_| self.is_required()).into_iter())
    }

    pub(crate) fn compare(&self, target: &WrapperTypes) -> WrapperTypesComparison {
        use WrapperType::*;
        use WrapperTypesComparison::*;

        let mut src_wrappers = self.iter_wrappers();
        let mut target_wrappers = target.iter_wrappers();
        let mut end_state = NoChange;

        loop {
            match (src_wrappers.next(), target_wrappers.next()) {
                (Some(List), Some(List))
                | (Some(RequiredList), Some(RequiredList))
                | (Some(Required), Some(Required)) => (),

                (Some(Required), None) | (Some(RequiredList), Some(List)) => {
                    end_state = match end_state {
                        NoChange | RemovedNonNull => RemovedNonNull,
                        AddedNonNull | NotCompatible => NotCompatible,
                    }
                }
                (None, Some(Required)) | (Some(List), Some(RequiredList)) => {
                    end_state = match end_state {
                        NoChange | AddedNonNull => AddedNonNull,
                        RemovedNonNull | NotCompatible => NotCompatible,
                    }
                }
                (Some(_), _) | (_, Some(_)) => break NotCompatible,

                (None, None) => break end_state,
            }
        }
    }

    pub(crate) fn set_required(&mut self, required: bool) {
        self.0 |= u64::from(required) << Self::REQUIRED_BIT_OFFSET;
    }

    pub(crate) fn push_list(&mut self, required: bool) {
        let lists_count = self.lists_count();

        if lists_count > Self::LIST_BITS_COUNT {
            // Too many list wrappers
            return;
        }

        if required {
            self.0 |= 1 << lists_count;
        }

        let new_lists_count = lists_count + 1;

        self.0 &= !Self::LIST_COUNT_BITS_MASK;
        self.0 |= new_lists_count << Self::LIST_COUNT_BITS_OFFSET;
    }
}

/// The relevant changes that can happen in wrapper types for the purposes of diffing.
#[derive(Debug, Clone, Copy)]
pub(crate) enum WrapperTypesComparison {
    NoChange,
    /// The type is not required anymore _at any level_
    RemovedNonNull,
    //// The type became required _at any level_
    AddedNonNull,
    /// List nesting level changed such that there exist values of src that will not fit in target
    /// and vice versa.
    NotCompatible,
}
