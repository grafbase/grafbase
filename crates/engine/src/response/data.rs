use schema::ObjectDefinitionId;

use crate::response::{ResponseField, ResponseFieldsSortedByKey};

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
        debug_assert!(reservation.is_empty(), "Part already has data");
        debug_assert_eq!(part.next_available_shared_fields_index, 0, "No dangling shared fields");
        debug_assert_eq!(part.next_available_list_index, 0, "No dangling shared list");
        debug_assert_eq!(part.next_available_map_index, 0, "No dangling shared map");
        *reservation = part;
    }

    pub fn view_object(&self, id: ResponseObjectId) -> (Option<ObjectDefinitionId>, &[ResponseField]) {
        let part = &self[id.part_id];
        let object = &part[id.object_id];
        let fields = match object.fields_sorted_by_key {
            ResponseFieldsSortedByKey::Slice {
                fields_id,
                offset,
                limit,
            } => {
                let start = offset as usize;
                let end = start + limit as usize;
                &part[fields_id][start..end]
            }
            ResponseFieldsSortedByKey::Owned { fields_id } => &part[fields_id],
        };
        (object.definition_id, fields)
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
    #[indexed_by(PartSharedFieldsId)]
    shared_fields: Vec<Vec<ResponseField>>,
    next_available_shared_fields_index: usize,
    #[indexed_by(PartOwnedFieldsId)]
    owned_fields: Vec<Vec<ResponseField>>,
    // Contrary to fields, we don't the shared/owned separation because we never
    // add or remove values from lists. When we push what would be an owned list we
    // still keep track of the offset & limit. So if we later extend we'll only read the relevant
    // part.
    #[indexed_by(PartListId)]
    lists: Vec<Vec<ResponseValue>>,
    next_available_list_index: usize,
    #[indexed_by(PartInaccesibleValueId)]
    inaccessible_values: Vec<ResponseValue>,
    #[indexed_by(PartMapId)]
    maps: Vec<Vec<(String, ResponseValue)>>,
    next_available_map_index: usize,
}

impl DataPart {
    pub(super) fn new(id: DataPartId) -> Self {
        Self {
            id,
            objects: Vec::new(),
            lists: Vec::new(),
            next_available_list_index: 0,
            shared_fields: Vec::new(),
            next_available_shared_fields_index: 0,
            owned_fields: Vec::new(),
            inaccessible_values: Vec::new(),
            maps: Vec::new(),
            next_available_map_index: 0,
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
                part_id,
                object_id,
                key,
                nullable,
            } => {
                debug_assert!(part_id == self.id && nullable, "{part_id} == {} && {nullable}", self.id);
                match self[object_id].fields_sorted_by_key {
                    ResponseFieldsSortedByKey::Slice {
                        fields_id,
                        offset,
                        limit,
                    } => {
                        let start = offset as usize;
                        let end = start + limit as usize;
                        match self[fields_id][start..end].binary_search_by(|probe| probe.key.cmp(&key)) {
                            Ok(index) => {
                                let mut inaccessible_value = ResponseValue::Inaccessible {
                                    id: ResponseInaccessibleValueId {
                                        part_id: self.id,
                                        value_id: PartInaccesibleValueId::from(self.inaccessible_values.len()),
                                    },
                                };
                                std::mem::swap(&mut self[fields_id][start + index].value, &mut inaccessible_value);
                                self.inaccessible_values.push(inaccessible_value);
                            }
                            Err(_) => {
                                unreachable!("Slice fields should always contain the inaccessible field");
                            }
                        }
                    }
                    ResponseFieldsSortedByKey::Owned {
                        fields_id: fields_list_id,
                    } => {
                        match self[fields_list_id].binary_search_by(|probe| probe.key.cmp(&key)) {
                            Ok(index) => {
                                let mut inaccessible_value = ResponseValue::Inaccessible {
                                    id: ResponseInaccessibleValueId {
                                        part_id: self.id,
                                        value_id: PartInaccesibleValueId::from(self.inaccessible_values.len()),
                                    },
                                };
                                std::mem::swap(&mut self[fields_list_id][index].value, &mut inaccessible_value);
                                self.inaccessible_values.push(inaccessible_value);
                            }
                            // May not be present for extension field resolver as they add fields directly,
                            // rather than entities.
                            Err(index) => {
                                self[fields_list_id].insert(
                                    index,
                                    ResponseField {
                                        key,
                                        value: ResponseValue::Null,
                                    },
                                );
                            }
                        }
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

    pub fn push_empty_object(&mut self, definition_id: Option<ObjectDefinitionId>) -> ResponseObjectId {
        self.push_object(ResponseObject {
            definition_id,
            fields_sorted_by_key: ResponseFieldsSortedByKey::Slice {
                fields_id: 0usize.into(),
                offset: 0,
                limit: 0,
            },
        })
    }

    pub fn put_object(&mut self, ResponseObjectId { part_id, object_id }: ResponseObjectId, object: ResponseObject) {
        debug_assert!(part_id == self.id);
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

    pub fn take_next_list(&mut self) -> (ResponseListId, Vec<ResponseValue>) {
        let i = self.next_available_list_index;
        self.next_available_list_index += 1;
        let list = if let Some(list) = self.lists.get_mut(i) {
            std::mem::take(list)
        } else {
            self.lists.push(Vec::new());
            Vec::new()
        };
        let id = ResponseListId {
            part_id: self.id,
            list_id: PartListId::from(i),
        };
        (id, list)
    }

    pub fn restore_list(&mut self, id: ResponseListId, list: Vec<ResponseValue>) {
        let i = usize::from(id.list_id);
        self.next_available_list_index -= 1;
        debug_assert!(id.part_id == self.id && i == self.next_available_list_index && i < self.lists.len());
        self.lists[i] = list;
    }

    pub fn push_owned_sorted_fields_by_key(&mut self, fields: Vec<ResponseField>) -> PartOwnedFieldsId {
        let fields_id = PartOwnedFieldsId::from(self.owned_fields.len());
        self.owned_fields.push(fields);
        fields_id
    }

    pub fn take_next_shared_fields(&mut self) -> (PartSharedFieldsId, Vec<ResponseField>) {
        let i = self.next_available_shared_fields_index;
        self.next_available_shared_fields_index += 1;
        let fields = if let Some(fields) = self.shared_fields.get_mut(i) {
            std::mem::take(fields)
        } else {
            self.shared_fields.push(Vec::new());
            Vec::new()
        };
        (PartSharedFieldsId::from(i), fields)
    }

    pub fn restore_shared_fields(&mut self, id: PartSharedFieldsId, fields: Vec<ResponseField>) {
        let i = usize::from(id);
        self.next_available_shared_fields_index -= 1;
        debug_assert!(i < self.shared_fields.len() && i == self.next_available_shared_fields_index);
        self.shared_fields[i] = fields;
    }

    pub fn take_next_map(&mut self) -> (ResponseMapId, Vec<(String, ResponseValue)>) {
        let i = self.next_available_map_index;
        self.next_available_map_index += 1;
        let map = if let Some(map) = self.maps.get_mut(i) {
            std::mem::take(map)
        } else {
            self.maps.push(Vec::new());
            Vec::new()
        };
        let id = ResponseMapId {
            part_id: self.id,
            map_id: PartMapId::from(i),
        };
        (id, map)
    }

    pub fn restore_map(&mut self, id: ResponseMapId, map: Vec<(String, ResponseValue)>) {
        let i = usize::from(id.map_id);
        self.next_available_map_index -= 1;
        debug_assert!(id.part_id == self.id && i == self.next_available_map_index && i < self.maps.len());
        self.maps[i] = map;
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
        write!(f, "OBJ#{}#{}", self.part_id.0, self.object_id.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartOwnedFieldsId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartSharedFieldsId(u16);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, id_derives::Id)]
pub(crate) struct PartListId(u16);

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
