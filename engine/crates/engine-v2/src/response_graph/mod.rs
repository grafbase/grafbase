use std::collections::BTreeMap;

use ::schema::ObjectId;
use schema::Schema;

mod de;
mod edges;
mod input;
mod selection_set;
mod ser;

pub use edges::{
    Argument, FieldEdge, FieldEdgeId, FieldName, Pos, ResponseGraphEdges, ResponseGraphEdgesBuilder, TypeCondition,
};
pub use input::Input;
pub use selection_set::{
    InputNodeSelection, InputNodeSelectionSet, NodeSelection, NodeSelectionSet, OutputNodeSelection,
    OutputNodeSelectionSet,
};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodePath(Vec<FieldEdgeId>);

struct NodeRelativePath(Vec<FieldEdgeId>);

impl NodePath {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn child(&self, field: FieldEdgeId) -> Self {
        let mut child = Self(self.0.clone());
        child.0.push(field);
        child
    }

    fn relative_to(&self, parent: &NodePath) -> Option<NodeRelativePath> {
        if self.0.starts_with(&parent.0) {
            Some(NodeRelativePath(self.0[parent.0.len()..].to_vec()))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ObjectNodeId(usize);

pub struct ResponseGraph {
    edges: ResponseGraphEdges,
    root: ObjectNodeId,
    object_nodes: Vec<ObjectNode>,
    object_node_ids_cache: BTreeMap<NodePath, Vec<ObjectNodeId>>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl ResponseGraph {
    pub fn new(edges: ResponseGraphEdges) -> Self {
        let root_node = ObjectNode {
            object_id: None,
            fields: vec![],
        };
        Self {
            edges,
            root: ObjectNodeId(0),
            object_nodes: vec![root_node],
            object_node_ids_cache: BTreeMap::new(),
        }
    }

    pub fn push_object(&mut self, object: ObjectNode) -> ObjectNodeId {
        self.object_nodes.push(object);
        ObjectNodeId(self.object_nodes.len() - 1)
    }

    pub fn input<'a>(
        &'a mut self,
        schema: &'a Schema,
        path: &'a NodePath,
        selection_set: &'a InputNodeSelectionSet,
    ) -> Option<Input<'a>> {
        self.insert_object_node_ids_in_cache(schema, path);
        let object_node_ids = self
            .object_node_ids_cache
            .get(path)
            .expect("object node ids have just been added to the cache");
        if object_node_ids.is_empty() {
            None
        } else {
            Some(Input {
                object_node_ids: self
                    .object_node_ids_cache
                    .get(path)
                    .expect("object node ids have just been added to the cache"),
                graph: self,
                selection_set,
            })
        }
    }

    fn insert_object_node_ids_in_cache(&mut self, schema: &Schema, path: &NodePath) {
        if self.object_node_ids_cache.contains_key(path) {
            return;
        }
        let object_node_ids = self
            .object_node_ids_cache
            // Longest paths first so we're finding the first closest cache path.
            .range(..path)
            .rev()
            .find_map(|(maybe_parent, object_node_ids)| {
                path.relative_to(maybe_parent)
                    .map(|relative_path| self.traverse(schema, object_node_ids, relative_path))
            })
            .expect("Root node id is part of the cache.");
        self.object_node_ids_cache.insert(path.clone(), object_node_ids);
    }

    fn traverse(&self, schema: &Schema, roots: &[ObjectNodeId], relative_path: NodeRelativePath) -> Vec<ObjectNodeId> {
        let mut out = vec![];

        let relative_path = relative_path
            .0
            .into_iter()
            .map(|edge_id| {
                let edge = &self.edges[edge_id];
                (edge.type_condition, edge.name)
            })
            .collect::<Vec<_>>();

        'outer: for root in roots {
            let mut object_node_id = *root;

            for (type_condition, name) in &relative_path {
                let node = &self[object_node_id];
                if let Some(type_condition) = type_condition {
                    let object_id = node
                        .object_id
                        .expect("Missing object_id on a node that is subject to a type condition.");
                    let type_matches = match type_condition {
                        TypeCondition::Interface(interface_id) => {
                            schema[object_id].implements_interfaces.contains(interface_id)
                        }
                        TypeCondition::Object(expected_object_id) => object_id == *expected_object_id,
                        TypeCondition::Union(union_id) => schema[*union_id].members.contains(&object_id),
                    };
                    if !type_matches {
                        continue 'outer;
                    }
                }

                if let Some(found) = node.field(*name).and_then(|node| node.as_object()) {
                    object_node_id = found;
                } else {
                    continue 'outer;
                }
            }

            out.push(object_node_id);
        }

        out
    }
}

impl std::ops::Index<FieldEdgeId> for ResponseGraph {
    type Output = FieldEdge;

    fn index(&self, index: FieldEdgeId) -> &Self::Output {
        &self.edges[index]
    }
}

impl std::ops::Index<ObjectNodeId> for ResponseGraph {
    type Output = ObjectNode;

    fn index(&self, index: ObjectNodeId) -> &Self::Output {
        &self.object_nodes[index.0]
    }
}

impl std::ops::IndexMut<ObjectNodeId> for ResponseGraph {
    fn index_mut(&mut self, index: ObjectNodeId) -> &mut Self::Output {
        &mut self.object_nodes[index.0]
    }
}

impl std::ops::Index<FieldName> for ResponseGraph {
    type Output = str;

    fn index(&self, index: FieldName) -> &Self::Output {
        &self.edges[index]
    }
}

pub struct ObjectNode {
    // object_id will only be present if __typename was retrieved which always be the case
    // through proper planning when it's needed for unions/interfaces between plans.
    object_id: Option<ObjectId>,
    fields: Vec<(FieldName, Node)>,
}

impl ObjectNode {
    pub fn insert_fields(&mut self, fields: impl IntoIterator<Item = (FieldName, Node)>) {
        self.fields.extend(fields);
    }

    pub fn field(&self, target: FieldName) -> Option<&Node> {
        self.fields
            .iter()
            .find_map(|(name, node)| if *name == target { Some(node) } else { None })
    }
}

pub enum Node {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String), // We should probably intern enums.
    List(Vec<Node>),
    Object(ObjectNodeId),
}

impl Node {
    fn as_object(&self) -> Option<ObjectNodeId> {
        match self {
            Self::Object(id) => Some(*id),
            _ => None,
        }
    }
}
