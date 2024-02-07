/// ResponseEdge is a single u32 with all the information bitpacked to have an effecient key
/// for the BTreeMap storing fields. It structured as follows:
///
///  0000_0000_0000_0000_0000_0000_0000_0000
///  ↑
///  BoundResponseKey flag
///
///  0 -> BoundResponseKey
///
///     ↓ Position (max 65_536) in the query, ensuring proper ordering of the response fields
///   ┌──────────────────┐
///  0000_0000_0000_0000_0000_0000_0000_0000
///                       └────────────────┘
///                         ↑ ResponseKey (max 32_768), interned string id of the response key
///  1 -> Other
///
///     0000_0000_0000_0000_0000_0000_0000_0000
///      ↑
///      Extra/Index flag
///
///      0 -> Extra
///
///     1000_0000_0000_0000_0000_0000_0000_0000
///                          └────────────────┘
///                            ↑ ResponseKey
///
///      1 -> Index
///
///     1100_0000_0000_0000_0000_0000_0000_0000
///       └───────────────────────────────────┘
///         ↑ Index (within a list)
///
/// The GraphQL spec requires that fields are orderd in the same order as the query. To keep track
/// of it, we bitpack the query position of each field with its ResponseKey (interned string id).
/// The Response stores all object fields in a BTreeMap, and with the position at the front, we
/// ensure proper order by iterating over the BTreeMap in order.
///
/// Additionally to BoundResponseKeys there are two other kinds of edges:
/// - extra fields: Fields added during the planning because a child plan required them. As they
///                 don't exist in the query they must not be send back. So we put them at the end,
///                 after any possible BoundResponseKey. We use the FieldId for the rest of the
///                 value to ensure its uniqueness and have a simpler field collection when merging
///                 selection sets.
/// - indices: Only used in ResponsePath to for errors. As the path is copied a lot bitpacking it
///            is a nice bonus.
///
/// Due to bitpacking we have the following constraints on the Query:
/// - At most 65_536 bound fields (after spreading named fragments)
/// - At most 32_768 different response keys
/// Which I consider to be a decent enough margin for any sane query. At worst we'll increase it if
/// really necessary.
const POSITION_MASK: u32 = 0b0111_1111_1111_1111_1000_0000_0000_0000;
const POSITION_BIT_SHIFT: u32 = POSITION_MASK.trailing_zeros();
const MAX_RESPONSE_KEY: u32 = 1 << POSITION_BIT_SHIFT;
const MAX_POSITION: u32 = POSITION_MASK >> POSITION_BIT_SHIFT;

const OTHER_FLAG: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

const EXTRA_FIELD_KEY_FLAG: u32 = OTHER_FLAG;
const EXTRA_FIELD_KEY_MASK: u32 = !EXTRA_FIELD_KEY_FLAG;

const INDEX_FLAG: u32 = OTHER_FLAG | 0b0100_0000_0000_0000_0000_0000_0000_0000;
const INDEX_MASK: u32 = !INDEX_FLAG;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseEdge(u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundResponseKey(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseKey(u32);

unsafe impl lasso::Key for ResponseKey {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    #[allow(clippy::panic)]
    fn try_from_usize(id: usize) -> Option<Self> {
        let id = u32::try_from(id).ok()?;
        if id <= MAX_RESPONSE_KEY {
            Some(Self(id))
        } else {
            panic!("Cannot create any new response keys!");
        }
    }
}

impl ResponseKey {
    pub fn with_position(self, position: usize) -> Option<BoundResponseKey> {
        u32::try_from(position).ok().and_then(|position| {
            if position <= MAX_POSITION {
                let key = self.0;
                // Sanity check we don't overlap
                assert!(
                    key < (1 << POSITION_BIT_SHIFT) && ((key & POSITION_MASK) == 0),
                    "response key is too big"
                );
                Some(BoundResponseKey(key | (position << POSITION_BIT_SHIFT)))
            } else {
                None
            }
        })
    }
}

impl BoundResponseKey {
    pub(crate) fn key(self) -> ResponseKey {
        ResponseKey(self.0 & !POSITION_MASK)
    }

    pub(crate) fn position(self) -> usize {
        (self.0 & POSITION_MASK) as usize >> POSITION_BIT_SHIFT
    }
}

pub enum UnpackedResponseEdge {
    Index(usize),
    BoundResponseKey(BoundResponseKey),
    ExtraFieldResponseKey(ResponseKey),
}

impl ResponseEdge {
    pub(crate) fn unpack(self) -> UnpackedResponseEdge {
        if self.0 & OTHER_FLAG == 0 {
            UnpackedResponseEdge::BoundResponseKey(BoundResponseKey(self.0))
        } else if self.0 & INDEX_FLAG == INDEX_FLAG {
            UnpackedResponseEdge::Index((self.0 & INDEX_MASK) as usize)
        } else {
            assert!(self.0 & EXTRA_FIELD_KEY_FLAG == EXTRA_FIELD_KEY_FLAG);
            UnpackedResponseEdge::ExtraFieldResponseKey(ResponseKey(self.0 & EXTRA_FIELD_KEY_MASK))
        }
    }
}

impl UnpackedResponseEdge {
    #[allow(clippy::panic)]
    pub(crate) fn pack(self) -> ResponseEdge {
        match self {
            UnpackedResponseEdge::BoundResponseKey(key) => {
                // Sanity check we don't overlap
                assert!(key.0 & OTHER_FLAG == 0);
                ResponseEdge(key.0)
            }
            UnpackedResponseEdge::Index(index) => {
                let index = index as u32;
                assert!((index & INDEX_MASK) == index, "Insufficient space for the index",);
                ResponseEdge(index | INDEX_FLAG)
            }
            UnpackedResponseEdge::ExtraFieldResponseKey(key) => {
                assert!(
                    (key.0 & EXTRA_FIELD_KEY_MASK) == key.0,
                    "Insufficient space for the key"
                );
                ResponseEdge(EXTRA_FIELD_KEY_FLAG | key.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lasso::Key;

    use super::*;

    #[test]
    fn field_name_value_in_range() {
        let key = ResponseKey::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = ResponseKey::try_from_usize(MAX_RESPONSE_KEY as usize).unwrap();
        assert_eq!(key.into_usize(), (MAX_RESPONSE_KEY as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = ResponseKey::try_from_usize((MAX_RESPONSE_KEY + 1) as usize);
        assert!(key.is_none());

        let key = ResponseKey::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
