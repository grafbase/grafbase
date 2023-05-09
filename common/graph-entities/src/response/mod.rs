//! `QueryResponse` is an AST which aims to represent a result of a `DynaQL` response.
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

use crate::CompactValue;
use core::fmt::{self, Display, Formatter};
use derivative::Derivative;
use dynaql_value::Name;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod entity_id;
mod into_response_node;
mod response_node_id;
mod se;

use self::response_node_id::ResponseNodeReference;

pub use self::{entity_id::EntityId, into_response_node::IntoResponseNode, response_node_id::ResponseNodeId};
pub use se::GraphQlResponseSerializer;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Root of the whole struct which is a Container
    root: Option<ResponseNodeId>,
    /// Storage of every nodes
    #[serde(with = "vectorize")]
    data: HashMap<ResponseNodeId, QueryResponseNode>,
    /// Map of database NodeId to the ID used in data
    entity_ids: HashMap<EntityId, ResponseNodeId>,
    /// The next id we can use when we add a node.
    next_id: u32,
}

pub mod vectorize {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter::FromIterator;

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        ser.collect_seq(target.into_iter())
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
    type Item = &'a QueryResponseNode;

    fn next(&mut self) -> Option<&'a QueryResponseNode> {
        let node = self.nodes.pop().and_then(|x| self.response.get_node(&x));
        match node {
            base @ Some(QueryResponseNode::Container(container)) => {
                container.children.iter().for_each(|(_, elt)| {
                    self.nodes.push(*elt);
                });
                base
            }
            base @ Some(QueryResponseNode::List(container)) => {
                container.children.iter().for_each(|elt| {
                    self.nodes.push(*elt);
                });
                base
            }
            base @ Some(QueryResponseNode::Primitive(_)) => base,
            None => None,
        }
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

    pub fn relations(&self) -> Relations<'_> {
        Relations {
            nodes: self.children(),
            relations: Vec::new(),
        }
    }
}

// TODO: iterator are little flawed right now as it's just a draft impl; it'll be switched to a
// more compact and efficient form later.
impl<'a> Iterator for Relations<'a> {
    type Item = (ResponseNodeRelation, EntityId);

    fn next(&mut self) -> Option<(ResponseNodeRelation, EntityId)> {
        loop {
            if let Some(relation) = self.relations.pop() {
                return Some(relation);
            }

            if let Some(node) = self.nodes.next() {
                match node {
                    QueryResponseNode::Container(container) => {
                        self.relations.extend(
                            container
                                .children
                                .iter()
                                .filter(|(rel, _)| matches!(rel, ResponseNodeRelation::Relation { .. }))
                                .filter_map(|(rel, _)| Some((rel.clone(), container.id.clone()?))),
                        );
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
pub struct Relations<'a> {
    nodes: Children<'a>,
    relations: Vec<(ResponseNodeRelation, EntityId)>,
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
        };
        this.new_node_unchecked(node);
        this
    }

    pub fn id_for_node<S: ResponseIdLookup>(&self, node: &S) -> Option<ResponseNodeId> {
        node.response_node_id(self)
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

    /// Create a new node, replace if the node already exist
    /// (A new node is not in the response)
    pub fn new_node_unchecked<T>(&mut self, node: T) -> ResponseNodeId
    where
        T: IntoResponseNode,
    {
        let next_id = self.next_id();
        self.new_node_unchecked_impl(node.entity_id(), node.into_node(), next_id)
    }

    fn new_node_unchecked_impl(
        &mut self,
        entity_id: Option<EntityId>,
        node: QueryResponseNode,
        node_id: ResponseNodeId,
    ) -> ResponseNodeId {
        if let Some(entity_id) = entity_id {
            if let Some(old_id) = self.entity_ids.insert(entity_id, node_id) {
                self.data.remove(&old_id);
            }
        }

        self.data.insert(node_id, node);
        node_id
    }

    /// Get a Node by his ID
    pub fn get_node<S: ResponseNodeReference>(&self, id: &S) -> Option<&QueryResponseNode> {
        self.data.get(&id.response_node_id(self)?)
    }

    /// Get a Node by his ID
    pub fn get_node_mut<S: ResponseNodeReference>(&mut self, id: &S) -> Option<&mut QueryResponseNode> {
        self.data.get_mut(&id.response_node_id(self)?)
    }

    /// Delete a Node by his ID
    pub fn delete_node<S: ResponseNodeReference>(&mut self, id: S) -> Result<QueryResponseNode, QueryResponseErrors> {
        let actual_id = id.response_node_id(self).ok_or(QueryResponseErrors::NodeNotFound)?;
        if let Some(entity_id) = id.entity_id() {
            self.entity_ids.remove(&entity_id);
        }
        self.data.remove(&actual_id).ok_or(QueryResponseErrors::NodeNotFound)
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
        let id = self.new_node_unchecked(to);
        let from_node = self.get_node_mut(&from_id).ok_or(QueryResponseErrors::NodeNotFound)?;

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
        let id = self.new_node_unchecked(to);
        let from_node = self.get_node_mut(&from_id).ok_or(QueryResponseErrors::NodeNotFound)?;

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
                .take_node_into_const_value(root_id)
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
    pub fn take_node_into_const_value(&mut self, node_id: ResponseNodeId) -> Option<CompactValue> {
        match self.delete_node(node_id).ok()? {
            QueryResponseNode::Container(container) => {
                let ResponseContainer { children, .. } = *container;
                let mut fields = Vec::with_capacity(children.len());

                for (relation, nested_id) in children {
                    match self.take_node_into_const_value(nested_id)? {
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
                    list.push(self.take_node_into_const_value(node)?);
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
        self.get_node(&id).is_some()
    }
}

impl QueryResponseNode {
    pub fn id(&self) -> Option<EntityId> {
        match self {
            QueryResponseNode::Container(value) => value.id.clone(),
            QueryResponseNode::List(_) | QueryResponseNode::Primitive(_) => None,
        }
    }

    pub fn is_node(&self) -> bool {
        matches!(self.id(), Some(_))
    }

    pub const fn is_list(&self) -> bool {
        matches!(self, QueryResponseNode::List(_))
    }

    pub const fn is_container_or_node(&self) -> bool {
        matches!(self, QueryResponseNode::Container(_))
    }

    pub fn child(&self, relation: &ResponseNodeRelation) -> Option<&ResponseNodeId> {
        self.children()?
            .iter()
            .find_map(|(key, child)| if key == relation { Some(child) } else { None })
    }

    pub fn child_mut(&mut self, relation: &ResponseNodeRelation) -> Option<&mut ResponseNodeId> {
        self.children_mut()?
            .iter_mut()
            .find_map(|(key, child)| if key == relation { Some(child) } else { None })
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
#[derive(Derivative, Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq)]
#[derivative(Hash, PartialEq)]
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
        #[derivative(Hash = "ignore", PartialEq = "ignore")]
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
            .find(|(existing_name, _)| *existing_name == name)
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

    use crate::NodeID;

    use super::*;

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
        response.delete_node(glossary_id).unwrap();
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
        response.delete_node(glossary_id).unwrap();
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
}
