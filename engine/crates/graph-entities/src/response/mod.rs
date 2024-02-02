//! `QueryResponse` is an AST which aims to represent a result of a `Engine` response.
//!
//! This structure is the **resolved** version of a query, the point is not to have a logic of any
//! kind considering graph, the point is to be able to have a representation of a answer where we
//! are able to remove and add elements **BY NODE ID** where it would be translated into JSON.
//!
//! # Why do we need that?
//!
//! The purpose of this structure is to be shared across multiple layers / application. It allow us
//! to have an abstraction between the result of a query and the final representation for the user.
//!
//! If we create the final representation directly, then we can't add any metadata in the response
//! for other services or application to use.
//!
//! For instance, live-queries are working by knowing what data is requested by an user, and
//! process every events hapenning on the database to identify if the followed data changed. If the
//! followed data changed, so it means the server will have to compute the diff between those. To
//! be able to faithfully compute the diff, it's much more simplier to not use the path of this
//! data but to use the unique ID of the data you are modifying. Hence, this representation.
//!
//! ### Serialization & Memory Use
//!
//! We've seen a lot of memory problems with this structure on larger query responses, so we need
//! to be careful to keep both the in memory size and serialization size down.  As a result most
//! of the types in this file have some serde attrs that make them more compact when serialized

use core::fmt::{self, Display, Formatter};
use std::collections::{HashMap, HashSet, VecDeque};

use engine_value::Name;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};

use crate::CompactValue;

mod entity_id;
mod into_response_node;
mod response_node_id;
mod se;

pub use se::GraphQlResponseSerializer;

use self::response_node_id::ToEntityId;
pub use self::{entity_id::EntityId, into_response_node::IntoResponseNode, response_node_id::ResponseNodeId};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Root of the whole struct which is a Container
    root: Option<ResponseNodeId>,
    /// Storage of every nodes
    #[serde(with = "vectorize")]
    data: HashMap<ResponseNodeId, QueryResponseNode>,
    /// Map of database NodeId to the ID used in the data
    ///
    /// Database entities can appear more than once in a response, so we need to keep a list here.
    entity_ids: HashMap<EntityId, Vec<ResponseNodeId>>,
    /// The next id we can use when we add a node.
    next_id: u32,
    /// Cache tags
    cache_tags: HashSet<String>,
}

pub mod vectorize {
    use std::iter::FromIterator;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        ser.collect_seq(target)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(container.into_iter().collect::<T>())
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = (ResponseNodeId, &'a QueryResponseNode);

    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.pop().and_then(|id| {
            self.response.get_node(id).map(|node| {
                match &node {
                    QueryResponseNode::Container(container) => {
                        container.children.iter().for_each(|(_, elt)| {
                            self.nodes.push(*elt);
                        });
                    }
                    QueryResponseNode::List(container) => {
                        container.children.iter().for_each(|elt| {
                            self.nodes.push(*elt);
                        });
                    }
                    _ => (),
                };
                (id, node)
            })
        })
    }
}
#[derive(Clone)]
/// An iterator of the IDs of the children of a given node
pub struct Children<'a> {
    response: &'a QueryResponse,
    nodes: Vec<ResponseNodeId>,
}

impl QueryResponse {
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    pub fn children(&self) -> Children<'_> {
        Children {
            response: self,
            nodes: if let Some(root) = &self.root {
                vec![*root]
            } else {
                Vec::new()
            },
        }
    }

    pub fn relations(&self) -> RelationsIterator<'_> {
        RelationsIterator {
            nodes: self.children(),
            items: VecDeque::new(),
        }
    }

    pub fn cache_tags(&self) -> &HashSet<String> {
        &self.cache_tags
    }
}

// TODO: iterator are little flawed right now as it's just a draft impl; it'll be switched to a
// more compact and efficient form later.
impl<'a> Iterator for RelationsIterator<'a> {
    type Item = RelationsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(relation) = self.items.pop_front() {
                return Some(relation);
            }

            if let Some((node_id, node)) = self.nodes.next() {
                match node {
                    QueryResponseNode::Container(container) => {
                        self.items.extend(container.children.iter().filter_map(|(rel, _)| {
                            if matches!(rel, ResponseNodeRelation::Relation { .. }) {
                                Some(RelationsIteratorItem {
                                    relation: rel.clone(),
                                    parent_node_id: node_id,
                                    parent_entity_id: container.id.clone(),
                                })
                            } else {
                                None
                            }
                        }));
                        continue;
                    }
                    _ => {
                        continue;
                    }
                }
            }

            return None;
        }
    }
}

/// An iterator of the IDs of the children of a given node with forward depth-first
pub struct RelationsIterator<'a> {
    nodes: Children<'a>,
    items: VecDeque<RelationsIteratorItem>,
}

pub struct RelationsIteratorItem {
    pub parent_node_id: ResponseNodeId,
    pub parent_entity_id: Option<EntityId>,
    pub relation: ResponseNodeRelation,
}

#[derive(Debug, thiserror::Error)]
pub enum QueryResponseErrors {
    #[error("The target node was not found")]
    NodeNotFound,
    #[error("The target node should be a container but wasn't")]
    NotAContainer,
    #[error("The target node should be a list but wasn't")]
    NotAList,
}

impl QueryResponse {
    /// Initialize a new response
    pub fn new_root<T>(node: T) -> Self
    where
        T: IntoResponseNode,
    {
        let id = ResponseNodeId(0);
        let mut this = Self {
            root: Some(id),
            entity_ids: HashMap::new(),
            data: HashMap::new(),
            next_id: 0,
            cache_tags: HashSet::new(),
        };
        this.insert_node(node);
        this
    }

    pub fn ids_for_entity<T: ToEntityId>(&self, entity: &T) -> Vec<ResponseNodeId> {
        self.entity_ids.get(&entity.entity_id()).cloned().unwrap_or_default()
    }

    fn next_id(&mut self) -> ResponseNodeId {
        let id = ResponseNodeId(self.next_id);
        self.next_id += 1;
        assert!(self.next_id != 0, "Oh no, an ID Overflow");
        id
    }

    /// Set the new root node
    pub fn set_root_unchecked(&mut self, id: ResponseNodeId) {
        self.root = Some(id);
    }

    /// Create a new node
    pub fn insert_node<T>(&mut self, node: T) -> ResponseNodeId
    where
        T: IntoResponseNode,
    {
        let entity_id = node.entity_id();
        let node_id = self.next_id();
        if let Some(entity_id) = entity_id {
            self.entity_ids.entry(entity_id).or_default().push(node_id);
        }

        self.data.insert(node_id, node.into_node());
        node_id
    }

    /// Get a Node by his ID
    pub fn get_node(&self, id: ResponseNodeId) -> Option<&QueryResponseNode> {
        self.data.get(&id)
    }

    /// Get a Node by his ID
    pub fn get_node_mut(&mut self, id: ResponseNodeId) -> Option<&mut QueryResponseNode> {
        self.data.get_mut(&id)
    }

    pub fn get_entity_nodes<T: ToEntityId>(&self, entity_id: &T) -> impl Iterator<Item = &QueryResponseNode> {
        self.ids_for_entity(entity_id)
            .into_iter()
            .filter_map(|node_id| self.data.get(&node_id))
    }

    /// Delete a Node by entity ID
    pub fn delete_entity<S: ToEntityId>(&mut self, id: S) -> Result<(), QueryResponseErrors> {
        let entity_id = id.entity_id();
        let node_ids = self
            .entity_ids
            .remove(&entity_id)
            .ok_or(QueryResponseErrors::NodeNotFound)?;

        let mut error = None;
        for id in node_ids {
            if self.data.remove(&id).is_none() {
                error = Some(QueryResponseErrors::NodeNotFound);
            }
        }

        match error {
            None => Ok(()),
            Some(error) => Err(error),
        }
    }

    // /// Delete a Node by node ID
    pub fn delete_node(&mut self, id: ResponseNodeId) -> Result<QueryResponseNode, QueryResponseErrors> {
        self.data.remove(&id).ok_or(QueryResponseErrors::NodeNotFound)
    }

    /// Append a new node to another node which has to be a `Container`
    /// replace if the node already exist
    pub fn append_unchecked<T>(
        &mut self,
        from_id: ResponseNodeId,
        to: T,
        relation: ResponseNodeRelation,
    ) -> Result<ResponseNodeId, QueryResponseErrors>
    where
        T: IntoResponseNode,
    {
        let id = self.insert_node(to);
        let from_node = self.get_node_mut(from_id).ok_or(QueryResponseErrors::NodeNotFound)?;

        if let QueryResponseNode::Container(container) = from_node {
            container.insert(relation, id);
        } else {
            return Err(QueryResponseErrors::NotAContainer);
        }

        Ok(id)
    }

    /// Push a new node to another node which has to be a `List`
    pub fn push<T>(&mut self, from_id: ResponseNodeId, to: T) -> Result<ResponseNodeId, QueryResponseErrors>
    where
        T: IntoResponseNode,
    {
        let id = self.insert_node(to);
        let from_node = self.get_node_mut(from_id).ok_or(QueryResponseErrors::NodeNotFound)?;

        if let QueryResponseNode::List(list) = from_node {
            list.children.push(id);
        } else {
            return Err(QueryResponseErrors::NotAContainer);
        }

        Ok(id)
    }

    pub fn into_compact_value(mut self) -> serde_json::Result<CompactValue> {
        Ok(match self.root {
            Some(root_id) => self
                .take_node_into_compact_value(root_id)
                .expect("graph root should always exist"),
            None => CompactValue::Object(Default::default()),
        })
    }

    /// Creates a serde_json::Value of the Response.
    ///
    /// The resulting serde_json::Value can take a lot of memory so
    /// serializing direct to a response should be preferred where possible.
    pub fn to_json_value(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self.as_graphql_data())
    }

    /// Removes a node and it's children from the Graph, and returns a CompactValue of its data.
    pub fn take_node_into_compact_value(&mut self, node_id: ResponseNodeId) -> Option<CompactValue> {
        match self.delete_node(node_id).ok()? {
            QueryResponseNode::Container(container) => {
                let ResponseContainer { children, .. } = *container;
                let mut fields = Vec::with_capacity(children.len());

                for (relation, nested_id) in children {
                    match self.take_node_into_compact_value(nested_id)? {
                        // Skipping nested empty objects
                        CompactValue::Object(fields) if fields.is_empty() => (),
                        value => {
                            fields.push((Name::new(relation.to_string()), value));
                        }
                    }
                }
                Some(CompactValue::Object(fields))
            }
            QueryResponseNode::List(list) => {
                let ResponseList { children, .. } = *list;
                let mut list = Vec::with_capacity(children.len());
                for node in children {
                    list.push(self.take_node_into_compact_value(node)?);
                }
                Some(CompactValue::List(list))
            }
            QueryResponseNode::Primitive(primitive) => {
                let ResponsePrimitive(value) = *primitive;
                Some(value)
            }
        }
    }

    fn node_exists(&self, id: ResponseNodeId) -> bool {
        self.get_node(id).is_some()
    }

    pub fn add_cache_tags<Tag: Into<String>>(&mut self, tags: impl IntoIterator<Item = Tag>) {
        self.cache_tags.extend(tags.into_iter().map(Into::into));
    }
}

impl QueryResponseNode {
    pub fn entity_id(&self) -> Option<EntityId> {
        match self {
            QueryResponseNode::Container(value) => value.id.clone(),
            QueryResponseNode::List(_) | QueryResponseNode::Primitive(_) => None,
        }
    }

    pub fn is_entity_node(&self) -> bool {
        self.entity_id().is_some()
    }

    pub const fn is_list(&self) -> bool {
        matches!(self, QueryResponseNode::List(_))
    }

    pub const fn is_container(&self) -> bool {
        matches!(self, QueryResponseNode::Container(_))
    }

    pub fn child(&self, relation: &ResponseNodeRelation) -> Option<&ResponseNodeId> {
        self.children()?.iter().find_map(|(key, child)| {
            if key.same_internal_field(relation) {
                Some(child)
            } else {
                None
            }
        })
    }

    pub fn child_mut(&mut self, relation: &ResponseNodeRelation) -> Option<&mut ResponseNodeId> {
        self.children_mut()?.iter_mut().find_map(|(key, child)| {
            if key.same_internal_field(relation) {
                Some(child)
            } else {
                None
            }
        })
    }

    pub fn children(&self) -> Option<&Vec<(ResponseNodeRelation, ResponseNodeId)>> {
        match self {
            Self::Container(container) => Some(&container.children),
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<(ResponseNodeRelation, ResponseNodeId)>> {
        match self {
            Self::Container(container) => Some(&mut container.children),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ResponseList {
    // Right now children are in an order based on the created_at which can be derived.
    // What we should do is to add a a OrderedBy field where we would specified Ord applied to this
    // List. Then on insert we'll be able to add new elements based on the Ord.
    // order: Vec<todo!()>,
    #[serde(rename = "c", default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<ResponseNodeId>,
}

impl ResponseList {
    pub fn with_children(children: Vec<ResponseNodeId>) -> Box<Self> {
        Box::new(Self {
            // id: ResponseNodeId::internal(),
            children,
        })
    }

    /// Element at the specified index
    pub fn insert(&mut self, index: usize, id: ResponseNodeId) {
        self.children.insert(index, id);
    }

    /// Push a new element into the `List` (at the end)
    pub fn push(&mut self, id: ResponseNodeId) {
        self.children.push(id);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponsePrimitive(CompactValue);

impl ResponsePrimitive {
    pub fn new(value: CompactValue) -> Box<Self> {
        Box::new(ResponsePrimitive(value))
    }

    pub fn is_null(&self) -> bool {
        matches!(self.0, CompactValue::Null)
    }
}

impl Default for ResponsePrimitive {
    fn default() -> Self {
        ResponsePrimitive(CompactValue::Null)
    }
}

/// This structure represent a link between two node, this can be a Relation when two node are
/// connected together or this can also be a `NotARelation`.
///
/// NB: `NotARelation` is hashed based on the field value **only**.
// temp: might be interesting to invest time to change it at the root level to have vertices
// flattened depending on the needs on the structure.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ResponseNodeRelation {
    #[serde(rename = "R")]
    Relation {
        #[serde(rename = "rk")]
        response_key: ArcIntern<String>,
        #[serde(rename = "rn")]
        relation_name: ArcIntern<String>,
        #[serde(rename = "f", default, skip_serializing_if = "Option::is_none")]
        from: Option<ArcIntern<String>>,
        #[serde(rename = "t")]
        to: ArcIntern<String>,
    },
    #[serde(rename = "NR")]
    NotARelation {
        #[serde(rename = "rk", default, skip_serializing_if = "Option::is_none")]
        response_key: Option<ArcIntern<String>>,
        #[serde(rename = "f")]
        field: ArcIntern<String>,
    },
}

impl ResponseNodeRelation {
    pub fn relation(response_key: String, relation_name: String, from: Option<String>, to: String) -> Self {
        Self::Relation {
            response_key: ArcIntern::new(response_key),
            relation_name: ArcIntern::new(relation_name),
            from: from.map(ArcIntern::new),
            to: ArcIntern::new(to.to_lowercase()),
        }
    }

    pub const fn not_a_relation(value: ArcIntern<String>, response_key: Option<ArcIntern<String>>) -> Self {
        Self::NotARelation {
            field: value,
            response_key,
        }
    }

    /// Returns true if self & other appear to represent the same field of a model/type (i.e. ignoring response_key)
    pub fn same_internal_field(&self, other: &ResponseNodeRelation) -> bool {
        match (self, other) {
            (
                ResponseNodeRelation::Relation {
                    relation_name: relation_name_lhs,
                    from: from_lhs,
                    to: to_lhs,
                    ..
                },
                ResponseNodeRelation::Relation {
                    relation_name: relation_name_rhs,
                    from: from_rhs,
                    to: to_rhs,
                    ..
                },
            ) => relation_name_lhs == relation_name_rhs && from_lhs == from_rhs && to_lhs == to_rhs,
            (
                ResponseNodeRelation::NotARelation { field: field_lhs, .. },
                ResponseNodeRelation::NotARelation { field: field_rhs, .. },
            ) => field_lhs == field_rhs,
            _ => false,
        }
    }

    fn response_key(&self) -> &str {
        match self {
            ResponseNodeRelation::Relation { response_key, .. }
            | ResponseNodeRelation::NotARelation {
                response_key: Some(response_key),
                ..
            } => response_key.as_str(),
            ResponseNodeRelation::NotARelation { field, .. } => field.as_str(),
        }
    }
}

impl Display for ResponseNodeRelation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ResponseNodeRelation::Relation { response_key, .. } => write!(f, "{response_key}"),
            ResponseNodeRelation::NotARelation { response_key, field } => {
                write!(f, "{}", response_key.as_ref().unwrap_or(field))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RelationOrigin {
    #[serde(rename = "N")]
    Node(ResponseNodeId),
    #[serde(rename = "T")]
    Type(ArcIntern<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseContainer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<EntityId>,

    /// Children which are (relation_name, node)
    #[serde(rename = "c")]
    children: Vec<(ResponseNodeRelation, ResponseNodeId)>,

    // /// Errors, not as `ServerError` yet as we do not have the position.
    // errors: Vec<Error>,
    /// # Hack
    ///
    /// temp: hack to have relation followed types, why this is a hack? because in fact we are doing
    /// something wrong with this abstraction: we modelize it like we would do for a json response
    /// with metadata, we shouldn't but we don't have the choice at first because it would imply to
    /// work on other parts too (execution step).
    /// For instance an "edge" node doesn't have any sense, nor does the pageInfo node too, these
    /// are intersting for the end result based on the request and the end result, but this
    /// representation either has to be agnostic of it, or the fact the the relation is followed
    /// should belong here.
    ///
    /// We'll need to think a little about it while working on the execution step.
    ///
    /// To have the system of following relation working, we need to store here relations that are
    /// OneToMany, and we need to follow the origin node (if any) or the origin type and the
    /// relation.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "r")]
    relation: Option<(RelationOrigin, ArcIntern<String>)>,
}

impl ResponseContainer {
    pub fn new_node(id: impl Into<EntityId>) -> Self {
        Self {
            id: Some(id.into()),
            children: Default::default(),
            relation: None,
            // errors: Vec::new(),
        }
    }

    pub fn new_container() -> Self {
        Self {
            id: None,
            children: Default::default(),
            relation: None,
            // errors: Vec::new(),
        }
    }

    pub fn set_relation(&mut self, rel: Option<(RelationOrigin, ArcIntern<String>)>) {
        self.relation = rel;
    }

    pub fn with_children(children: impl IntoIterator<Item = (ResponseNodeRelation, ResponseNodeId)>) -> Self {
        Self {
            id: None,
            children: children.into_iter().collect(),
            relation: None,
            // errors: Vec::new(),
        }
    }

    /// Insert a new node with a relation, if an Old Node was present, the Old node will be
    /// replaced
    pub fn insert(&mut self, name: ResponseNodeRelation, mut node: ResponseNodeId) -> Option<ResponseNodeId> {
        if let Some((_, existing)) = self
            .children
            .iter_mut()
            .find(|(existing_relation, _)| existing_relation.response_key() == name.response_key())
        {
            std::mem::swap(existing, &mut node);
            return Some(node);
        }
        self.children.push((name, node));
        None
    }
}

/// A Query Response Node
#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum QueryResponseNode {
    #[serde(rename = "C")]
    Container(Box<ResponseContainer>),
    #[serde(rename = "L")]
    List(Box<ResponseList>),
    #[serde(rename = "P")]
    Primitive(Box<ResponsePrimitive>),
}

#[cfg(test)]
mod tests {
    use internment::ArcIntern;
    use serde_json::Number;

    use super::*;
    use crate::NodeID;

    #[test]
    fn check_size_of_query_response_node() {
        // Each node of the response graph gets a QueryResponseNode.  These graphs can
        // get big (230k nodes in a large introspection query) so we need to keep
        // QueryResponseNode as small as possible to avoid running out of memory.
        assert_eq!(std::mem::size_of::<QueryResponseNode>(), 16);
        assert_eq!(std::mem::size_of::<ResponseNodeId>(), 4);

        // TODO: Can I make this smaller?
        assert_eq!(std::mem::size_of::<ResponseContainer>(), 56);

        assert_eq!(std::mem::size_of::<ResponseNodeRelation>(), 32);
    }

    #[test]
    fn should_transform_into_simple_json() {
        let primitive_node = ResponsePrimitive::new(CompactValue::String("blbl".into()));
        let response = QueryResponse::new_root(primitive_node);

        assert_eq!(
            response.to_json_value().unwrap().to_string(),
            serde_json::json!("blbl").to_string()
        );
    }

    #[test]
    fn should_transform_example_json() {
        let root = ResponseContainer::new_container();
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let glossary_container = response
            .append_unchecked(
                root_id,
                ResponseContainer::new_container(),
                ResponseNodeRelation::NotARelation {
                    response_key: None,
                    field: "glossary".to_string().into(),
                },
            )
            .unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));
        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "title".to_string().into(),
        };

        response
            .append_unchecked(glossary_container, example_primitive, relation)
            .unwrap();

        let output_json = serde_json::json!({
            "glossary": {
                "title": "example",
            }
        });

        assert_eq!(response.to_json_value().unwrap().to_string(), output_json.to_string());
    }

    #[test]
    fn should_have_float_as_float() {
        let root = ResponseContainer::new_container();
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let glossary_container = response
            .append_unchecked(
                root_id,
                ResponseContainer::new_container(),
                ResponseNodeRelation::NotARelation {
                    response_key: None,
                    field: "glossary".to_string().into(),
                },
            )
            .unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::Number(Number::from_f64(123.0).unwrap()));
        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "age".to_string().into(),
        };

        response
            .append_unchecked(glossary_container, example_primitive, relation)
            .unwrap();

        let output_json = serde_json::json!({
            "glossary": {
                "age": 123.0,
            }
        });

        assert_eq!(response.to_json_value().unwrap().to_string(), output_json.to_string());
    }

    #[test]
    fn should_be_able_to_delete_a_node() {
        let root = ResponseContainer::new_container();
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let glossary_id = NodeID::new("type", "a_id");
        let glossary_container = response
            .append_unchecked(
                root_id,
                ResponseContainer::new_node(&glossary_id),
                ResponseNodeRelation::NotARelation {
                    response_key: None,
                    field: "glossary".to_string().into(),
                },
            )
            .unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));

        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "title".to_string().into(),
        };

        response
            .append_unchecked(glossary_container, example_primitive, relation)
            .unwrap();

        let output_json = serde_json::json!({
            "glossary": {
                "title": "example",
            }
        });

        assert_eq!(response.to_json_value().unwrap().to_string(), output_json.to_string());
        response.delete_entity(glossary_id).unwrap();
        assert_eq!(response.to_json_value().unwrap().to_string(), "{}");
    }

    #[test]
    fn delete_list_json() {
        let root = ResponseList::with_children(Vec::new());
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let node = response.push(root_id, ResponseContainer::new_container()).unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));

        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "test".to_string().into(),
        };
        response
            .append_unchecked(node, example_primitive.clone(), relation)
            .unwrap();

        let glossary_id = NodeID::new("type", "a_id");
        let glossary_container = response
            .push(root_id, ResponseContainer::new_node(&glossary_id))
            .unwrap();

        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "title".to_string().into(),
        };

        response
            .append_unchecked(glossary_container, example_primitive, relation)
            .unwrap();

        let output_json = serde_json::Value::Array(vec![
            serde_json::json!({
                "test": "example"
            }),
            serde_json::json!({
                "title": "example"
            }),
        ]);

        assert_eq!(response.to_json_value().unwrap().to_string(), output_json.to_string());
        response.delete_entity(glossary_id).unwrap();
        assert_eq!(
            response.to_json_value().unwrap().to_string(),
            "[{\"test\":\"example\"}]"
        );
    }

    #[test]
    fn transform_list_json() {
        let root = ResponseList::with_children(Vec::new());
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let node = response.push(root_id, ResponseContainer::new_container()).unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));

        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "test".to_string().into(),
        };

        response.append_unchecked(node, example_primitive, relation).unwrap();

        let output_json = serde_json::Value::Array(vec![serde_json::json!({
            "test": "example"
        })]);

        assert_eq!(response.to_json_value().unwrap().to_string(), output_json.to_string());
    }

    #[test]
    fn print_list_json() {
        let root = ResponseList::with_children(Vec::new());
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let node = response.push(root_id, ResponseContainer::new_container()).unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));

        let example_primitive_enum = ResponsePrimitive::new(CompactValue::Enum(ArcIntern::new("example".to_owned())));

        let relation = ResponseNodeRelation::NotARelation {
            response_key: None,
            field: "test".to_string().into(),
        };

        response.append_unchecked(node, example_primitive, relation).unwrap();

        response.push(root_id, example_primitive_enum).unwrap();

        let output = response.to_json_value().unwrap().to_string();

        let output_json = serde_json::Value::Array(vec![
            serde_json::json!({
                "test": "example"
            }),
            serde_json::json!("example"),
        ]);

        assert_eq!(output, output_json.to_string());
    }

    #[test]
    fn test_insert_node_duplicates_if_same_entity_id() {
        let node_id = NodeID::new_owned("todo".into(), ulid::Ulid::new().to_string());
        let container = ResponseContainer::new_node(node_id.clone());

        let mut response = QueryResponse::default();

        let id_one = response.insert_node(container.clone());
        let id_two = response.insert_node(container);

        assert_ne!(id_one, id_two);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.entity_ids.len(), 1);
    }

    #[test]
    fn test_insert_node_handles_different_entity_ids() {
        let node_id = NodeID::new_owned("todo".into(), ulid::Ulid::new().to_string());
        let node = ResponseContainer::new_node(node_id.clone());

        let node_id_two = NodeID::new_owned("todo".into(), ulid::Ulid::new().to_string());
        let node_two = ResponseContainer::new_node(node_id_two.clone());

        let mut response = QueryResponse::default();

        let id_one = response.insert_node(node);
        let id_two = response.insert_node(node_two);

        assert!(id_one != id_two);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.entity_ids.len(), 2);
    }
}
