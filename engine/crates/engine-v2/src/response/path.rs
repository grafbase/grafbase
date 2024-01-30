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
///       └───────────────────────────────────┘
///         ↑ ResponseKey, created during planning, so could be over 32_768.
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
const RESPONSE_KEY_MASK: u32 = 0b0000_0000_0000_0000_0111_1111_1111_1111;
const POSITION_MASK: u32 = 0b0111_1111_1111_1111_1000_0000_0000_0000;
const POSITION_BIT_SHIFT: u32 = RESPONSE_KEY_MASK.trailing_ones();
const MAX_POSITION: u32 = (1 << (POSITION_MASK.count_ones() + 1)) - 1;
const OTHER_FLAG: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
const EXTRA_FIELD_FLAG: u32 = OTHER_FLAG;
const INDEX_FLAG: u32 = OTHER_FLAG | 0b0100_0000_0000_0000_0000_0000_0000_0000;
const OTHER_DATA_MASK: u32 = 0b0011_1111_1111_1111_1111_1111_1111_1111;

#[derive(Default, Debug, Clone)]
pub struct ResponsePath(im::Vector<ResponseEdge>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseEdge(u32);

pub enum UnpackedResponseEdge {
    Index(usize),
    BoundResponseKey(BoundResponseKey),
    ExtraField(ResponseKey),
}

impl UnpackedResponseEdge {
    #[allow(clippy::panic)]
    pub fn pack(self) -> ResponseEdge {
        match self {
            UnpackedResponseEdge::Index(index) => {
                let index = index as u32;
                if index > OTHER_DATA_MASK {
                    panic!("Index is too high.");
                }
                ResponseEdge(index | INDEX_FLAG)
            }
            UnpackedResponseEdge::BoundResponseKey(key) => ResponseEdge(key.0),
            UnpackedResponseEdge::ExtraField(key) => ResponseEdge(EXTRA_FIELD_FLAG | key.0),
        }
    }
}

impl ResponseEdge {
    pub fn unpack(self) -> UnpackedResponseEdge {
        if self.0 & OTHER_FLAG == 0 {
            UnpackedResponseEdge::BoundResponseKey(BoundResponseKey(self.0 & !OTHER_FLAG))
        } else if self.0 & !OTHER_DATA_MASK == INDEX_FLAG {
            UnpackedResponseEdge::Index((self.0 & OTHER_DATA_MASK) as usize)
        } else {
            UnpackedResponseEdge::ExtraField(ResponseKey(self.0 & OTHER_DATA_MASK))
        }
    }

    pub fn is_extra(&self) -> bool {
        self.0 & !OTHER_DATA_MASK == EXTRA_FIELD_FLAG
    }

    pub fn as_response_key(&self) -> Option<ResponseKey> {
        match self.unpack() {
            UnpackedResponseEdge::BoundResponseKey(key) => Some(key.into()),
            UnpackedResponseEdge::ExtraField(key) => Some(key),
            _ => None,
        }
    }
}

impl ResponsePath {
    pub fn child(&self, segment: impl Into<ResponseEdge>) -> ResponsePath {
        let mut path = self.0.clone();
        path.push_back(segment.into());
        ResponsePath(path)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResponseEdge> {
        self.0.iter()
    }
}

impl From<BoundResponseKey> for ResponseEdge {
    fn from(value: BoundResponseKey) -> Self {
        UnpackedResponseEdge::BoundResponseKey(value).pack()
    }
}

impl From<usize> for ResponseEdge {
    #[allow(clippy::panic)]
    fn from(index: usize) -> Self {
        UnpackedResponseEdge::Index(index).pack()
    }
}

#[derive(Debug, Clone)]
pub struct ResponseKeys(lasso::Rodeo<ResponseKey>);

impl Default for ResponseKeys {
    fn default() -> Self {
        Self(lasso::Rodeo::new())
    }
}

impl ResponseKeys {
    pub fn get_or_intern(&mut self, s: &str) -> ResponseKey {
        self.0.get_or_intern(s)
    }

    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }

    pub fn try_resolve(&self, key: ResponseKey) -> Option<&str> {
        self.0.try_resolve(&key)
    }
}

impl std::ops::Index<BoundResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: BoundResponseKey) -> &Self::Output {
        self.0.resolve(&ResponseKey::from(key))
    }
}

impl std::ops::Index<ResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: ResponseKey) -> &Self::Output {
        self.0.resolve(&key)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseKey(u32);

impl ResponseKey {
    pub fn with_position(self, position: usize) -> Option<BoundResponseKey> {
        u32::try_from(position).ok().and_then(|position| {
            if position < MAX_POSITION {
                Some(BoundResponseKey(self.0 | (position << POSITION_BIT_SHIFT)))
            } else {
                None
            }
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundResponseKey(u32);

impl std::fmt::Debug for BoundResponseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position = (self.0 & POSITION_MASK) >> POSITION_BIT_SHIFT;
        let key = self.0 & RESPONSE_KEY_MASK;
        f.debug_struct("BoundResponseKey")
            .field("position", &position)
            .field("key", &key)
            .finish()
    }
}

impl From<BoundResponseKey> for ResponseKey {
    fn from(key: BoundResponseKey) -> Self {
        ResponseKey(key.0 & RESPONSE_KEY_MASK)
    }
}

impl From<&BoundResponseKey> for ResponseKey {
    fn from(key: &BoundResponseKey) -> Self {
        ResponseKey(key.0 & RESPONSE_KEY_MASK)
    }
}

unsafe impl lasso::Key for ResponseKey {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn try_from_usize(id: usize) -> Option<Self> {
        let id = u32::try_from(id).ok()?;
        if id <= RESPONSE_KEY_MASK {
            Some(Self(id))
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
        let key = ResponseKey::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = ResponseKey::try_from_usize(RESPONSE_KEY_MASK as usize).unwrap();
        assert_eq!(key.into_usize(), (RESPONSE_KEY_MASK as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = ResponseKey::try_from_usize((RESPONSE_KEY_MASK + 1) as usize);
        assert!(key.is_none());

        let key = ResponseKey::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
