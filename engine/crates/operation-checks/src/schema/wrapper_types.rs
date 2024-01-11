#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum WrapperType {
    List = 0b10,
    Required = 0b01,
    RequiredList = 0b11,
}

impl WrapperType {
    const LIST: u8 = WrapperType::List as u8;
    const REQUIRED: u8 = WrapperType::Required as u8;
    const REQUIRED_LIST: u8 = WrapperType::RequiredList as u8;
}

#[derive(Debug, PartialEq, Eq)]
pub enum WrapperTypes {
    /// A compact representation for up to four levels of wrapper types. Each pair of bits
    /// corresponds to an Option<WrapperType> (0b00 = None, 0b01 = Some(Required), 0b10 =>
    /// Some(List), 0b11 => Some(RequiredList)). The least significant pair of bits corresponds to
    /// the outermost wrappers.
    Small(u8),
    /// This variant is for extreme cases of list fields with more than three levels of nested
    /// lists. From outermost to innermost.
    Large(Box<[WrapperType]>),
}

impl WrapperTypes {
    /// Iterate wrapper types from outermost to innermost.
    pub(crate) fn iter_wrappers(&self) -> impl Iterator<Item = WrapperType> + '_ {
        let (first_four, rest): ([Option<WrapperType>; 4], &[WrapperType]) = match self {
            WrapperTypes::Small(small) => (
                [0, 1, 2, 3].map(move |i| {
                    let shift = i * 2; // we deal with groups of two bits

                    // Shift the bits we are interested in to the lower end of the byte and mask
                    // the rest away.
                    let bits = (small >> shift) & 0b11;

                    match bits {
                        0b00 => None,
                        WrapperType::REQUIRED => Some(WrapperType::Required),
                        WrapperType::LIST => Some(WrapperType::List),
                        WrapperType::REQUIRED_LIST => Some(WrapperType::RequiredList),
                        _ => unreachable!(),
                    }
                }),
                &[],
            ),
            WrapperTypes::Large(large) => ([None; 4], large.as_ref()),
        };

        first_four.into_iter().flatten().chain(rest.iter().copied())
    }

    pub(crate) fn is_required(&self) -> bool {
        matches!(
            self.iter_wrappers().next(),
            Some(WrapperType::Required | WrapperType::RequiredList)
        )
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
