use bytes::Bytes;

use crate::response::ResponseObjectField;

use super::{ResponseObject, ResponseValue, ResponseValueId};

/// Final representation of the response data after request execution.
pub(crate) struct ResponseData {
    pub(super) root: ResponseObjectId,
    pub(super) parts: DataParts,
}

impl std::ops::Deref for ResponseData {
    type Target = DataParts;
    fn deref(&self) -> &Self::Target {
        &self.parts
    }
}

impl ResponseData {
    pub(super) fn root_object(&self) -> &ResponseObject {
        &self.parts[self.root]
    }
}

/// The response data is composed of multiple parts, each with its own objects and lists.
/// This allows subgraph request to be processed independently. Each object/list is uniquely
/// identifier by its DataPartId and PartObjectId/PartListId.
#[derive(Default)]
pub(crate) struct DataParts(Vec<DataPart>);

impl DataParts {
    pub(super) fn new_part(&mut self) -> DataPart {
        let id = DataPartId(self.0.len() as u16);
        // reserving the spot until the actual data is written. It's safe as no one can reference
        // any data in this part before it's added. And a part can only be overwritten if it's
        // empty.
        self.0.push(DataPart::new(id));
        DataPart::new(id)
    }

    pub(super) fn insert(&mut self, part: DataPart) {
        let reservation = &mut self[part.id];
        assert!(reservation.is_empty(), "Part already has data");
        *reservation = part;
    }
}

impl std::ops::Index<DataPartId> for DataParts {
    type Output = DataPart;
    fn index(&self, index: DataPartId) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl std::ops::IndexMut<DataPartId> for DataParts {
    fn index_mut(&mut self, index: DataPartId) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

impl std::ops::Index<ResponseInaccessibleValueId> for DataParts {
    type Output = ResponseValue;
    fn index(&self, index: ResponseInaccessibleValueId) -> &Self::Output {
        &self[index.part_id][index.value_id]
    }
}

impl std::ops::IndexMut<ResponseInaccessibleValueId> for DataParts {
    fn index_mut(&mut self, index: ResponseInaccessibleValueId) -> &mut Self::Output {
        &mut self[index.part_id][index.value_id]
    }
}

impl std::ops::Index<ResponseObjectId> for DataParts {
    type Output = ResponseObject;
    fn index(&self, index: ResponseObjectId) -> &Self::Output {
        &self[index.part_id][index.object_id]
    }
}

impl std::ops::IndexMut<ResponseObjectId> for DataParts {
    fn index_mut(&mut self, index: ResponseObjectId) -> &mut Self::Output {
        &mut self[index.part_id][index.object_id]
    }
}

impl std::ops::Index<ResponseListId> for DataParts {
    type Output = [ResponseValue];
    fn index(&self, index: ResponseListId) -> &Self::Output {
        &self[index.part_id][index.list_id]
    }
}

impl std::ops::IndexMut<ResponseListId> for DataParts {
    fn index_mut(&mut self, index: ResponseListId) -> &mut Self::Output {
        &mut self[index.part_id][index.list_id]
    }
}

impl std::ops::Index<ResponseMapId> for DataParts {
    type Output = [(String, ResponseValue)];
    fn index(&self, index: ResponseMapId) -> &Self::Output {
        &self[index.part_id][index.map_id]
    }
}

impl std::ops::IndexMut<ResponseMapId> for DataParts {
    fn index_mut(&mut self, index: ResponseMapId) -> &mut Self::Output {
        &mut self[index.part_id][index.map_id]
    }
}

impl std::ops::Index<PartString> for DataParts {
    type Output = str;
    fn index(&self, s: PartString) -> &Self::Output {
        self[s.part_id].deref_part_string(s)
    }
}

// Not an id_derives as no one beside this file should need to create this ID.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct DataPartId(u16);

impl std::fmt::Display for DataPartId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part#{}", self.0)
    }
}

impl std::fmt::Debug for DataPartId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part#{}", self.0)
    }
}

#[derive(id_derives::IndexedFields)]
pub(crate) struct DataPart {
    pub id: DataPartId,
    bytes: Vec<Bytes>,
    strings: Vec<String>,
    #[indexed_by(PartObjectId)]
    objects: Vec<ResponseObject>,
    #[indexed_by(PartListId)]
    lists: Vec<Vec<ResponseValue>>,
    #[indexed_by(PartInaccesibleValueId)]
    inaccessible_values: Vec<ResponseValue>,
    #[indexed_by(PartMapId)]
    maps: Vec<Vec<(String, ResponseValue)>>,
}

impl DataPart {
    pub(super) fn new(id: DataPartId) -> Self {
        Self {
            id,
            bytes: Vec::new(),
            strings: Vec::new(),
            objects: Vec::new(),
            lists: Vec::new(),
            inaccessible_values: Vec::new(),
            maps: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.lists.is_empty()
    }

    // Any deserialization that use those bytes will be able to keep references for strings rather
    // than create new owned strings.
    pub fn push_borrowable_bytes(&mut self, bytes: Bytes) {
        self.bytes.push(bytes);
    }

    /// In a non-federated GraphQL server, values are simply set to null when propagating nulls for
    /// errors. It's not as simple in a federated context because the supergraph may need
    /// fields for subgraph requests or custom directives like `@authorized`. Let's take an example
    /// with the following query:
    ///
    /// ```graphql,ignore
    /// {
    ///     author { name }
    ///     posts { id }
    /// }
    /// ```
    ///
    /// and this schema:
    ///
    /// ```graphql,ignore
    /// type Author {
    ///   id: ID!
    ///   name: String!
    /// }
    ///
    /// type Post {
    ///   id: ID!
    /// }
    ///
    /// type Query {
    ///     author: Author @join__field(graph: A)
    ///     posts: [Post!]! @requires(fields: "author { id }") @join__field(graph: B)
    /// }
    /// ```
    ///
    /// If we have an error for `name`, we still need to be able to read the `id`
    /// field for the subgraph request retrieving `posts`. If we simply propagated null upwards, we
    /// would lose `author { id }` we retrieved for the `posts`. So instead we mark fields as inaccessible.
    /// During serialization of the response to the client those are treated as null. For every other
    /// purpose inaccessible fields are transparent.
    ///
    /// This happens when the supergraph needs to propagate a null for an `@inaccessible` field we detect at runtime,
    /// which can occur for enum values or inaccessible objects we may encounter behind an interface/union.
    pub fn make_inaccessible(&mut self, value_id: ResponseValueId) {
        match value_id {
            ResponseValueId::Field {
                part_id,
                object_id,
                key,
                nullable,
            } => {
                debug_assert!(part_id == self.id && nullable, "{part_id} == {} && {nullable}", self.id);
                match self[object_id]
                    .fields_sorted_by_key
                    .binary_search_by(|probe| probe.key.cmp(&key))
                {
                    Ok(index) => {
                        let mut inaccessible_value = ResponseValue::Inaccessible {
                            id: ResponseInaccessibleValueId {
                                part_id: self.id,
                                value_id: PartInaccesibleValueId::from(self.inaccessible_values.len()),
                            },
                        };
                        std::mem::swap(
                            &mut self[object_id].fields_sorted_by_key[index].value,
                            &mut inaccessible_value,
                        );
                        self.inaccessible_values.push(inaccessible_value);
                    }
                    // May not be present for extension field resolver as they add fields directly,
                    // rather than entities.
                    Err(index) => {
                        self[object_id].fields_sorted_by_key.insert(
                            index,
                            ResponseObjectField {
                                key,
                                value: ResponseValue::Null,
                            },
                        );
                    }
                }
            }
            ResponseValueId::Index {
                part_id,
                list_id,
                index,
                nullable,
            } => {
                debug_assert!(part_id == self.id && nullable, "{part_id} == {} && {nullable}", self.id);
                let mut inaccessible_value = ResponseValue::Inaccessible {
                    id: ResponseInaccessibleValueId {
                        part_id: self.id,
                        value_id: PartInaccesibleValueId::from(self.inaccessible_values.len()),
                    },
                };
                std::mem::swap(&mut self[list_id][index as usize], &mut inaccessible_value);
                self.inaccessible_values.push(inaccessible_value);
            }
        }
    }

    pub fn push_inaccessible_value(&mut self, value: ResponseValue) -> ResponseInaccessibleValueId {
        let value_id = PartInaccesibleValueId::from(self.inaccessible_values.len());
        self.inaccessible_values.push(value);
        ResponseInaccessibleValueId {
            part_id: self.id,
            value_id,
        }
    }

    pub fn push_object(&mut self, object: ResponseObject) -> ResponseObjectId {
        let object_id = PartObjectId::from(self.objects.len());
        self.objects.push(object);
        ResponseObjectId {
            part_id: self.id,
            object_id,
        }
    }

    pub fn reserve_object_id(&mut self) -> ResponseObjectId {
        self.push_object(ResponseObject::new(None, Vec::new()))
    }

    pub fn put_object(&mut self, ResponseObjectId { part_id, object_id }: ResponseObjectId, object: ResponseObject) {
        debug_assert!(part_id == self.id && self[object_id].fields_sorted_by_key.is_empty());
        self[object_id] = object;
    }

    pub fn push_list(&mut self, list: Vec<ResponseValue>) -> ResponseListId {
        let list_id = PartListId::from(self.lists.len());
        self.lists.push(list);
        ResponseListId {
            part_id: self.id,
            list_id,
        }
    }

    pub fn reserve_list_id(&mut self) -> ResponseListId {
        self.push_list(Vec::new())
    }

    pub fn put_list(&mut self, ResponseListId { part_id, list_id }: ResponseListId, list: Vec<ResponseValue>) {
        debug_assert!(part_id == self.id && self[list_id].is_empty());
        self[list_id] = list;
    }

    pub fn push_map(&mut self, map: Vec<(String, ResponseValue)>) -> ResponseMapId {
        let map_id = PartMapId::from(self.maps.len());
        self.maps.push(map);
        ResponseMapId {
            part_id: self.id,
            map_id,
        }
    }

    pub fn push_string(&mut self, s: String) -> PartString {
        let len = s.len() as u32;
        let ptr = PartStrPtr(s.as_ptr());
        self.strings.push(s);
        let out = PartString {
            part_id: self.id,
            ptr,
            len,
        };
        debug_assert!(self.deref_part_string(out) == self.strings[self.strings.len() - 1]);
        out
    }

    pub fn push_borrowed_str(&mut self, s: &str) -> PartString {
        let out = if self.can_be_borrowed(s) {
            let len = s.len() as u32;
            let ptr = PartStrPtr(s.as_ptr());
            PartString {
                part_id: self.id,
                ptr,
                len,
            }
        } else {
            self.push_string(s.to_owned())
        };
        debug_assert!(self.deref_part_string(out) == s);
        out
    }

    fn can_be_borrowed(&self, s: &str) -> bool {
        let Some(bytes) = self.bytes.last() else {
            return false;
        };
        let bytes_range = bytes.as_ptr_range();
        let str_range = s.as_bytes().as_ptr_range();
        bytes_range.start <= str_range.start && str_range.end <= bytes_range.end
    }

    fn deref_part_string(&self, PartString { part_id, ptr, len }: PartString) -> &str {
        debug_assert!(self.id == part_id, "Mismatched DataPartId");
        let ptr = ptr.0;
        let len = len as usize;
        let end = unsafe { ptr.add(len) };
        debug_assert!(
            self.bytes.iter().any(|bytes| {
                let bytes_range = bytes.as_ptr_range();
                bytes_range.start <= ptr && end <= bytes_range.end
            }) || self.strings.iter().any(|s| {
                let s_bytes = s.as_bytes();
                let s_range = s_bytes.as_ptr_range();
                s_range.start <= ptr && end <= s_range.end
            })
        );
        // SAFETY: We ensured we were the ones building this PartString.
        //         PartStrPtr is only constructed from from a &str / String so it's valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartInaccesibleValueId(u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct ResponseInaccessibleValueId {
    pub part_id: DataPartId,
    pub value_id: PartInaccesibleValueId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartObjectId(u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct ResponseObjectId {
    pub part_id: DataPartId,
    pub object_id: PartObjectId,
}

impl std::fmt::Display for ResponseObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID#{}#{}", self.part_id.0, self.object_id.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartListId(u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct ResponseListId {
    pub part_id: DataPartId,
    pub list_id: PartListId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartMapId(u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct ResponseMapId {
    pub part_id: DataPartId,
    pub map_id: PartMapId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PartString {
    part_id: DataPartId,
    ptr: PartStrPtr,
    len: u32,
}

impl From<PartString> for ResponseValue {
    fn from(s: PartString) -> Self {
        Self::String {
            part_id: s.part_id,
            ptr: s.ptr,
            len: s.len,
        }
    }
}

impl PartString {
    pub unsafe fn new(part_id: DataPartId, ptr: PartStrPtr, len: u32) -> Self {
        Self { part_id, ptr, len }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PartStrPtr(*const u8);

// SAFETY: PartString is a pointer to a String and is never processed in any other way.
unsafe impl Send for PartStrPtr {}
unsafe impl Sync for PartStrPtr {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_string_creates_valid_part_string() {
        let mut part = DataPart::new(DataPartId(0));
        let test_string = "Hello, World!".to_string();
        let part_string = part.push_string(test_string.clone());

        assert_eq!(part.deref_part_string(part_string), "Hello, World!");
        assert_eq!(part_string.len, 13);
    }

    #[test]
    fn push_string_handles_empty_string() {
        let mut part = DataPart::new(DataPartId(0));
        let part_string = part.push_string(String::new());

        assert_eq!(part.deref_part_string(part_string), "");
    }

    #[test]
    fn push_string_handles_unicode() {
        let mut part = DataPart::new(DataPartId(0));
        let unicode_string = "こんにちは🦀".to_string();
        let part_string = part.push_string(unicode_string.clone());

        assert_eq!(part.deref_part_string(part_string), "こんにちは🦀");
    }

    #[test]
    fn push_borrowed_str_without_bytes_creates_owned_string() {
        let mut part = DataPart::new(DataPartId(0));
        let test_str = "Test string";
        let part_string = part.push_borrowed_str(test_str);

        assert_eq!(part.deref_part_string(part_string), "Test string");
    }

    #[test]
    fn push_borrowed_str_with_bytes_creates_borrowed_reference() {
        let mut part = DataPart::new(DataPartId(0));

        // Create bytes that contain our string
        let bytes = Bytes::from(r#"{"name": "John", "age": 30}"#);
        part.push_borrowable_bytes(bytes.clone());

        // Extract a substring that exists within the bytes
        let part_string = part.push_borrowed_str(str::from_utf8(&bytes[10..14]).unwrap());

        assert_eq!(part.deref_part_string(part_string), "John");
        assert!(part.strings.is_empty());
    }

    #[test]
    fn push_borrowed_str_falls_back_to_owned_when_not_in_bytes() {
        let mut part = DataPart::new(DataPartId(0));

        // Add some bytes
        part.push_borrowable_bytes(Bytes::from("some data"));

        // Try to borrow a string that's not within those bytes
        let external_str = "external string";
        let part_string = part.push_borrowed_str(external_str);

        assert_eq!(part.deref_part_string(part_string), "external string");
    }

    #[test]
    fn push_borrowable_bytes_stores_bytes() {
        let mut part = DataPart::new(DataPartId(0));

        let bytes1 = Bytes::from("first chunk");
        let bytes2 = Bytes::from("second chunk");

        part.push_borrowable_bytes(bytes1.clone());
        part.push_borrowable_bytes(bytes2.clone());

        assert_eq!(part.bytes.len(), 2);
        assert_eq!(part.bytes[0], bytes1);
        assert_eq!(part.bytes[1], bytes2);
    }

    #[test]
    fn push_borrowable_bytes_enables_borrowing_from_last_bytes() {
        let mut part = DataPart::new(DataPartId(0));

        let bytes = Bytes::from("This is test data for borrowing");
        part.push_borrowable_bytes(bytes.clone());

        // Borrow a substring from the actual bytes
        let bytes_str = std::str::from_utf8(&bytes).unwrap();
        let borrowed = part.push_borrowed_str(&bytes_str[8..12]); // "test"

        assert_eq!(part.deref_part_string(borrowed), "test");
        assert!(part.strings.is_empty()); // Should not create owned string
    }

    #[test]
    fn deref_part_string_returns_correct_string() {
        let mut part = DataPart::new(DataPartId(0));

        let s1 = part.push_string("First".to_string());
        let s2 = part.push_string("Second".to_string());
        let s3 = part.push_string("Third".to_string());

        assert_eq!(part.deref_part_string(s1), "First");
        assert_eq!(part.deref_part_string(s2), "Second");
        assert_eq!(part.deref_part_string(s3), "Third");
    }

    #[test]
    fn deref_part_string_handles_multiple_strings() {
        let mut part = DataPart::new(DataPartId(0));

        let strings = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];

        let part_strings: Vec<_> = strings.iter().map(|s| part.push_string(s.to_string())).collect();

        for (i, part_string) in part_strings.iter().enumerate() {
            assert_eq!(part.deref_part_string(*part_string), strings[i]);
        }
    }

    #[test]
    fn can_be_borrowed_correctly_identifies_borrowable_strings() {
        let mut part = DataPart::new(DataPartId(0));

        let bytes = Bytes::from("JSON: {\"key\": \"value\"}");
        part.push_borrowable_bytes(bytes.clone());

        // Get the actual slice from the bytes we stored
        let bytes_str = std::str::from_utf8(&bytes).unwrap();

        // Should be borrowable (within the bytes)
        assert!(part.can_be_borrowed(&bytes_str[7..20])); // {"key": "val

        // Should not be borrowable (external string)
        assert!(!part.can_be_borrowed("external"));
    }

    #[test]
    fn multiple_bytes_chunks_allow_borrowing_from_last() {
        let mut part = DataPart::new(DataPartId(0));

        let bytes1 = Bytes::from("First chunk of data");
        let bytes2 = Bytes::from("Second chunk of data");

        part.push_borrowable_bytes(bytes1.clone());
        part.push_borrowable_bytes(bytes2.clone());

        // Create slices from the actual bytes to test borrowing
        let bytes2_str = std::str::from_utf8(&bytes2).unwrap();
        let bytes1_str = std::str::from_utf8(&bytes1).unwrap();

        // Can borrow from the second chunk (last bytes)
        assert!(part.can_be_borrowed(&bytes2_str[7..12])); // "chunk"

        // Cannot borrow from the first chunk anymore (only last bytes are checked)
        assert!(!part.can_be_borrowed(&bytes1_str[0..5])); // "First"
    }

    #[test]
    fn cannot_be_borrowed_if_outside_the_source() {
        let mut part = DataPart::new(DataPartId(0));

        let full_bytes = Bytes::from("AAAA BBBB CCCC DDDD EEEE FFFF");
        let full_str = std::str::from_utf8(&full_bytes).unwrap();

        let middle_slice = full_bytes.slice(4..24);
        let middle_str = std::str::from_utf8(&middle_slice).unwrap();
        assert_eq!(middle_str, " BBBB CCCC DDDD EEEE");
        part.push_borrowable_bytes(middle_slice.clone());

        // Should be able to borrow from within the slice
        assert!(part.can_be_borrowed(&middle_str[0..4])); // "CCCC"
        assert!(part.can_be_borrowed(&middle_str[5..9])); // "DDDD"

        // Should NOT be able to borrow from outside the slice, even if the lengths would fit
        // This tests that we're checking actual pointer ranges, not just lengths
        assert!(!part.can_be_borrowed(&full_str[..4])); // "AAAA" - before the slice
        assert!(part.can_be_borrowed(&full_str[4..24]));
        assert!(!part.can_be_borrowed(&full_str[24..])); // "FFFF" - after the slice

        // Edge case: string at the exact boundaries
        assert!(part.can_be_borrowed(&middle_str[0..14])); // entire slice content
    }
}
