/// ResponseEdge is a single u32 with all the information bitpacked to have an effecient key
/// for the BTreeMap storing response fields. It structured as follows:
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
/// Additionally to BoundResponseKeys which are actual query fields specified by the user, there
/// are two other kinds of edges:
/// - extra fields: Fields added during the planning because a child plan required them. As they
///                 don't exist in the query they must not be send back. So we put them at the end,
///                 after any possible BoundResponseKey.
/// - indices: Used in ResponsePath to for errors.
///
/// Due to bitpacking we have the following constraints on the Query:
/// - At most 65_536 bound fields (after spreading named fragments)
/// - At most 32_768 different response keys
/// Which I consider to be a decent enough margin for any sane query. At worst we'll increase it if
/// really necessary.
const POSITION_MASK: u32 = 0b0111_1111_1111_1111_1000_0000_0000_0000;
const POSITION_BIT_SHIFT: u32 = POSITION_MASK.trailing_zeros();
const MAX_RESPONSE_KEY: u32 = (1 << POSITION_BIT_SHIFT) - 1;
const RESPONSE_KEY_MASK: u32 = MAX_RESPONSE_KEY;
const MAX_POSITION: u32 = POSITION_MASK >> POSITION_BIT_SHIFT;

const OTHER_FLAG: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

const EXTRA_FIELD_KEY_FLAG: u32 = OTHER_FLAG;
const EXTRA_FIELD_KEY_MASK: u32 = !EXTRA_FIELD_KEY_FLAG;

const INDEX_FLAG: u32 = OTHER_FLAG | 0b0100_0000_0000_0000_0000_0000_0000_0000;
const INDEX_MASK: u32 = !INDEX_FLAG;

mod private;
pub use private::*;

/// A ResponseEdge correspond to any edge within the response graph, so a field or an index.
/// It's the primary abstraction for the ResponsePath, and used at different places for simplicity.
/// Like BounResponseKey, it keeps the ordering of the fields. Indices and extra fields are put at
/// the back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseEdge(u32);

/// A ResponseKey associated with a position within the query, guaranteeing the right order of
/// fields in the output as we BTreeMaps to store them in the response.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundResponseKey(u32);

/// Id of an interned string within ResponseKeys.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseKey(u32);

impl SafeResponseKey {
    pub fn with_position(self, position: usize) -> Option<BoundResponseKey> {
        u32::try_from(position).ok().and_then(|position| {
            if position <= MAX_POSITION {
                Some(BoundResponseKey(u32::from(self) | (position << POSITION_BIT_SHIFT)))
            } else {
                None
            }
        })
    }
}

impl ResponseEdge {
    pub fn is_extra(&self) -> bool {
        matches!(self.unpack(), UnpackedResponseEdge::ExtraFieldResponseKey(_))
    }

    pub fn as_response_key(&self) -> Option<ResponseKey> {
        match self.unpack() {
            UnpackedResponseEdge::BoundResponseKey(key) => Some(key.as_response_key()),
            UnpackedResponseEdge::ExtraFieldResponseKey(key) => Some(key),
            _ => None,
        }
    }
}

impl BoundResponseKey {
    pub(crate) fn position(self) -> usize {
        (self.0 & POSITION_MASK) as usize >> POSITION_BIT_SHIFT
    }

    pub(crate) fn as_response_key(self) -> ResponseKey {
        ResponseKey(self.0 & RESPONSE_KEY_MASK)
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
                let key = key.0;
                assert!((key & EXTRA_FIELD_KEY_MASK) == key, "Insufficient space for the key");
                ResponseEdge(EXTRA_FIELD_KEY_FLAG | key)
            }
        }
    }
}

impl From<UnpackedResponseEdge> for ResponseEdge {
    fn from(value: UnpackedResponseEdge) -> Self {
        value.pack()
    }
}

impl std::fmt::Debug for BoundResponseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundResponseKey")
            .field("position", &self.position())
            .field("key", &self.as_response_key().0)
            .finish()
    }
}
