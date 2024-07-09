//! `QueryResponse` is an AST which aims to represent a result of a `Engine` response.
//!
//! ### Serialization & Memory Use
//!
//! We've seen a lot of memory problems with this structure on larger query responses, so we need
//! to be careful to keep both the in memory size and serialization size down.  As a result most
//! of the types in this file have some serde attrs that make them more compact when serialized

use std::collections::{HashMap, HashSet};

use engine_value::Name;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};

use crate::CompactValue;

mod into_response_node;
mod response_node_id;
mod se;

pub use se::GraphQlResponseSerializer;

pub use self::{into_response_node::IntoResponseNode, response_node_id::ResponseNodeId};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Root of the whole struct which is a Container
    pub root: Option<ResponseNodeId>,
    /// Storage of every nodes
    #[serde(with = "vectorize")]
    data: HashMap<ResponseNodeId, QueryResponseNode>,
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
                        container.0.iter().for_each(|(_, elt)| {
                            self.nodes.push(*elt);
                        });
                    }
                    QueryResponseNode::List(container) => {
                        container.0.iter().for_each(|elt| {
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

    pub fn cache_tags(&self) -> &HashSet<String> {
        &self.cache_tags
    }

    pub fn merge(&mut self, source_container_id: ResponseNodeId, destination_container_id: ResponseNodeId) {
        // We need to merge the contents of node_id into existing_id assuming they are both Containers
        let entries_to_append = self.get_node(source_container_id).and_then(|node| match node {
            QueryResponseNode::Container(container) => Some(container.iter().cloned().collect::<Vec<_>>()),
            _ => None,
        });

        let mut nested_merge_ids = vec![];

        if let Some((QueryResponseNode::Container(existing_container), entries_to_append)) =
            self.get_node_mut(destination_container_id).zip(entries_to_append)
        {
            for (name, src_field_id) in entries_to_append {
                if let Some(dest_field_id) = existing_container.child(name.as_str()) {
                    nested_merge_ids.push((src_field_id, dest_field_id));
                    continue;
                }
                existing_container.insert(name.as_str(), src_field_id);
            }
        }

        for (src_id, dest_id) in nested_merge_ids {
            self.merge(src_id, dest_id);
        }
    }
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
            data: HashMap::new(),
            next_id: 0,
            cache_tags: HashSet::new(),
        };
        this.insert_node(node);
        this
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
        let node_id = self.next_id();

        self.data.insert(node_id, node.into_node());
        node_id
    }

    /// Get a Node by his ID
    pub fn get_node(&self, id: ResponseNodeId) -> Option<&QueryResponseNode> {
        self.data.get(&id)
    }

    pub fn get_container_node(&self, id: ResponseNodeId) -> Option<&ResponseContainer> {
        match self.data.get(&id)? {
            QueryResponseNode::Container(container) => Some(container),
            _ => None,
        }
    }

    pub fn get_container_node_mut(&mut self, id: ResponseNodeId) -> Option<&mut ResponseContainer> {
        match self.data.get_mut(&id)? {
            QueryResponseNode::Container(container) => Some(container),
            _ => None,
        }
    }

    /// Get a Node by his ID
    pub fn get_node_mut(&mut self, id: ResponseNodeId) -> Option<&mut QueryResponseNode> {
        self.data.get_mut(&id)
    }

    /// Delete a Node by node ID
    pub fn delete_node(&mut self, id: ResponseNodeId) -> Result<QueryResponseNode, QueryResponseErrors> {
        self.data.remove(&id).ok_or(QueryResponseErrors::NodeNotFound)
    }

    /// Append a new node to another node which has to be a `Container`
    /// replace if the node already exist
    pub fn append_unchecked<T>(
        &mut self,
        from_id: ResponseNodeId,
        to: T,
        field: &str,
    ) -> Result<ResponseNodeId, QueryResponseErrors>
    where
        T: IntoResponseNode,
    {
        let id = self.insert_node(to);
        let from_node = self.get_node_mut(from_id).ok_or(QueryResponseErrors::NodeNotFound)?;

        if let QueryResponseNode::Container(container) = from_node {
            container.insert(field, id);
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
            list.0.push(id);
        } else {
            return Err(QueryResponseErrors::NotAContainer);
        }

        Ok(id)
    }

    pub fn into_compact_value(mut self) -> Option<CompactValue> {
        Some(
            self.take_node_into_compact_value(self.root?)
                .expect("graph root should always exist"),
        )
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
                let ResponseContainer(children) = *container;
                let mut fields = Vec::with_capacity(children.len());

                for (name, nested_id) in children {
                    let value = self.take_node_into_compact_value(nested_id)?;
                    fields.push((Name::new(name.to_string()), value));
                }
                Some(CompactValue::Object(fields))
            }
            QueryResponseNode::List(list) => {
                let ResponseList(children) = *list;
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
    pub const fn is_list(&self) -> bool {
        matches!(self, QueryResponseNode::List(_))
    }

    pub const fn is_container(&self) -> bool {
        matches!(self, QueryResponseNode::Container(_))
    }

    pub fn children(&self) -> Option<&Vec<(ArcIntern<String>, ResponseNodeId)>> {
        match self {
            Self::Container(container) => Some(&container.0),
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<(ArcIntern<String>, ResponseNodeId)>> {
        match self {
            Self::Container(container) => Some(&mut container.0),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ResponseList(Vec<ResponseNodeId>);

impl ResponseList {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = ResponseNodeId> + '_ {
        self.0.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_children(children: Vec<ResponseNodeId>) -> Box<Self> {
        Box::new(Self(children))
    }

    /// Element at the specified index
    pub fn insert(&mut self, index: usize, id: ResponseNodeId) {
        self.0.insert(index, id);
    }

    /// Push a new element into the `List` (at the end)
    pub fn push(&mut self, id: ResponseNodeId) {
        self.0.push(id);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponsePrimitive(pub CompactValue);

impl ResponsePrimitive {
    pub fn new(value: CompactValue) -> Box<Self> {
        Box::new(ResponsePrimitive(value))
    }

    pub fn is_null(&self) -> bool {
        matches!(self.0, CompactValue::Null)
    }

    pub fn is_string(&self) -> bool {
        matches!(self.0, CompactValue::String(_))
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.0 {
            CompactValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl Default for ResponsePrimitive {
    fn default() -> Self {
        ResponsePrimitive(CompactValue::Null)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseContainer(
    /// Children which are (field_name, node_id)
    Vec<(ArcIntern<String>, ResponseNodeId)>,
);

impl ResponseContainer {
    pub fn new_container() -> Self {
        Self(Default::default())
    }

    pub fn with_children(children: impl IntoIterator<Item = (ArcIntern<String>, ResponseNodeId)>) -> Self {
        Self(children.into_iter().collect())
    }

    /// Insert a new node with the given name.  If the name was already present it will be replaced.
    pub fn insert(&mut self, name: &str, node: ResponseNodeId) {
        if let Some((_, existing)) = self
            .0
            .iter_mut()
            .find(|(existing_name, _)| existing_name.as_str() == name)
        {
            *existing = node;
            return;
        }
        self.0.push((ArcIntern::new(name.to_string()), node));
    }

    pub fn child(&self, needle: &str) -> Option<ResponseNodeId> {
        let (_, id) = self.0.iter().find(|(name, _)| name.as_str() == needle)?;

        Some(*id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(ArcIntern<String>, ResponseNodeId)> {
        self.0.iter()
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

impl QueryResponseNode {
    pub fn as_container(&self) -> Option<&ResponseContainer> {
        match self {
            QueryResponseNode::Container(container) => Some(container.as_ref()),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        let QueryResponseNode::Primitive(primitive) = self else {
            return None;
        };
        primitive.as_str()
    }
}

#[cfg(test)]
mod tests {
    use internment::ArcIntern;
    use serde_json::Number;

    use super::*;

    #[test]
    fn check_size_of_query_response_node() {
        // Each node of the response graph gets a QueryResponseNode.  These graphs can
        // get big (230k nodes in a large introspection query) so we need to keep
        // QueryResponseNode as small as possible to avoid running out of memory.
        assert_eq!(std::mem::size_of::<QueryResponseNode>(), 16);
        assert_eq!(std::mem::size_of::<ResponseNodeId>(), 4);

        assert_eq!(std::mem::size_of::<ResponseContainer>(), 24);
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
            .append_unchecked(root_id, ResponseContainer::new_container(), "glossary")
            .unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));

        response
            .append_unchecked(glossary_container, example_primitive, "title")
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
            .append_unchecked(root_id, ResponseContainer::new_container(), "glossary")
            .unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::Number(Number::from_f64(123.0).unwrap()));

        response
            .append_unchecked(glossary_container, example_primitive, "age")
            .unwrap();

        let output_json = serde_json::json!({
            "glossary": {
                "age": 123.0,
            }
        });

        assert_eq!(response.to_json_value().unwrap().to_string(), output_json.to_string());
    }

    #[test]
    fn transform_list_json() {
        let root = ResponseList::with_children(Vec::new());
        let mut response = QueryResponse::new_root(root);
        let root_id = response.root.unwrap();

        let node = response.push(root_id, ResponseContainer::new_container()).unwrap();

        let example_primitive = ResponsePrimitive::new(CompactValue::String("example".to_string()));

        response.append_unchecked(node, example_primitive, "test").unwrap();

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

        response.append_unchecked(node, example_primitive, "test").unwrap();

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
