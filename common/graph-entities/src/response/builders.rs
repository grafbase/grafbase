use crate::{
    CompactValue, NodeID, RelationOrigin, ResponseContainer, ResponseList, ResponseNodeId, ResponseNodeRelation,
    ResponsePrimitive,
};
use internment::ArcIntern;

use super::QueryResponseNode;

#[derive(Debug)]
/// A builder for a ResponseContainer.
///
/// We need this separate struct as we need the
pub struct ResponseContainerBuilder {
    pub(super) entity_id: Option<ArcIntern<String>>,
    pub(super) children: Vec<(ResponseNodeRelation, ResponseNodeId)>,
    pub(super) relation: Option<(RelationOrigin, ArcIntern<String>)>,
}

// TODO: Actually wondering if we even need this separate type...

impl ResponseContainerBuilder {
    pub fn new_node<'a, S: AsRef<NodeID<'a>>>(id: S) -> Self {
        Self {
            entity_id: Some(ArcIntern::new(id.as_ref().to_string())),
            children: Default::default(),
            relation: None,
            // errors: Vec::new(),
        }
    }

    pub fn new_container() -> Self {
        Self {
            entity_id: None,
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
            entity_id: None,
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

pub trait ResponseNodeBuilder {
    fn entity_id(&self) -> Option<ArcIntern<String>>;
    fn into_node(self) -> QueryResponseNode;
}

impl ResponseNodeBuilder for Box<ResponsePrimitive> {
    fn entity_id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(self)
    }
}

impl ResponseNodeBuilder for Box<ResponseList> {
    fn entity_id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        crate::QueryResponseNode::List(self)
    }
}

impl ResponseNodeBuilder for ResponseContainerBuilder {
    fn entity_id(&self) -> Option<ArcIntern<String>> {
        self.entity_id.clone()
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Container(Box::new(ResponseContainer {
            id: self.entity_id,
            children: self.children,
            relation: self.relation,
        }))
    }
}

impl ResponseNodeBuilder for CompactValue {
    fn entity_id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(ResponsePrimitive::new(self))
    }
}
