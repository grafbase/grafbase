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
        let id = DataPartId::from(self.0.len());
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
        &self.0[usize::from(index)]
    }
}

impl std::ops::IndexMut<DataPartId> for DataParts {
    fn index_mut(&mut self, index: DataPartId) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct DataPartId(u16);

#[derive(id_derives::IndexedFields)]
pub(crate) struct DataPart {
    pub id: DataPartId,
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
            objects: Vec::new(),
            lists: Vec::new(),
            inaccessible_values: Vec::new(),
            maps: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.lists.is_empty()
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
                object_id: ResponseObjectId { part_id, object_id },
                query_position,
                response_key,
                nullable,
            } => {
                let key = response_key.with_position(query_position);
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
                list_id: ResponseListId { part_id, list_id },
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
