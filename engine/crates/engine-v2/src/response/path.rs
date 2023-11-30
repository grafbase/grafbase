// There isn't a lot of margin, but paying the cost of 4 additional bytes for every feels a bit
// excessive and the numbers are still huge for any sane query. So let's try. Easy to change
// otherwise.
// We support at most 32_768 different response key strings
const STRING_ID_MASK: u32 = 0b0000_0000_0000_0000_0111_1111_1111_1111;
const POSITION_BIT_SHIFT: u32 = STRING_ID_MASK.trailing_ones();
// We support at most 65_535 different bound fields (after spreading fragments).
// The last one is reserved for internal fields (inputs not present in the query) leaving them at
// the end of the response fields which is ordered by the bound response key. We don't really care
// about their ordering and having the same response key (string part) is all that actually
// matters.
const POSITION_MASK: u32 = 0b0111_1111_1111_1111_1000_0000_0000_0000;
// Using a single bit to differentiate between an index and a key
const INDEX_FLAG: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
const MAX_POSITION: u32 = (1 << 16) - 1;

#[derive(Default, Debug, Clone)]
pub struct ResponsePath(im::Vector<ResponsePathSegment>);

#[derive(Debug, Clone, Copy)]
pub struct ResponsePathSegment(u32);

impl ResponsePathSegment {
    pub fn try_into_bound_response_key(self) -> Result<BoundResponseKey, usize> {
        if self.0 & INDEX_FLAG == INDEX_FLAG {
            // 1-indexed
            // https://spec.graphql.org/October2021/#sec-Errors.Error-result-format
            let n = (self.0 & !INDEX_FLAG) + 1;
            Err(n as usize)
        } else {
            Ok(BoundResponseKey(self.0))
        }
    }
}

impl ResponsePath {
    pub fn child(&self, segment: impl Into<ResponsePathSegment>) -> ResponsePath {
        let mut path = self.0.clone();
        path.push_back(segment.into());
        ResponsePath(path)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResponsePathSegment> {
        self.0.iter()
    }
}

impl From<BoundResponseKey> for ResponsePathSegment {
    fn from(value: BoundResponseKey) -> Self {
        ResponsePathSegment(value.0)
    }
}

impl From<usize> for ResponsePathSegment {
    fn from(value: usize) -> Self {
        ResponsePathSegment((value as u32) | INDEX_FLAG)
    }
}

#[derive(Debug, Clone)]
pub struct ResponseKeys(lasso::Rodeo<ResponseKey>);

impl ResponseKeys {
    pub fn new() -> Self {
        Self(lasso::Rodeo::new())
    }

    pub fn get_or_intern(&mut self, s: &str) -> ResponseKey {
        self.0.get_or_intern(s)
    }
}

impl std::ops::Index<ResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, index: ResponseKey) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::Index<BoundResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, index: BoundResponseKey) -> &Self::Output {
        &self.0[ResponseKey(index.0 & STRING_ID_MASK)]
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
        let key = self.0 & STRING_ID_MASK;
        f.debug_struct("BoundResponseKey")
            .field("position", &position)
            .field("key", &key)
            .finish()
    }
}

impl BoundResponseKey {
    pub fn is_internal(self) -> bool {
        self.0 & POSITION_MASK == POSITION_MASK
    }
}

impl From<BoundResponseKey> for ResponseKey {
    fn from(key: BoundResponseKey) -> Self {
        ResponseKey(key.0 & STRING_ID_MASK)
    }
}

impl From<&BoundResponseKey> for ResponseKey {
    fn from(key: &BoundResponseKey) -> Self {
        ResponseKey(key.0 & STRING_ID_MASK)
    }
}

unsafe impl lasso::Key for ResponseKey {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn try_from_usize(id: usize) -> Option<Self> {
        let id = u32::try_from(id).ok()?;
        if id <= STRING_ID_MASK {
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

        let key = ResponseKey::try_from_usize(STRING_ID_MASK as usize).unwrap();
        assert_eq!(key.into_usize(), (STRING_ID_MASK as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = ResponseKey::try_from_usize((STRING_ID_MASK + 1) as usize);
        assert!(key.is_none());

        let key = ResponseKey::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
