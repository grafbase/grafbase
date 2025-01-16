use operation::Location;
use petgraph::{graph::NodeIndex, visit::EdgeRef};
use schema::CompositeTypeId;
use walker::Walk;

use crate::{FieldFlags, QueryField, QueryFieldNode, SpaceEdge, SpaceNode};

use super::QuerySolutionSpaceBuilder;

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn add_any_necessary_typename_fields(&mut self) -> crate::Result<()> {
        struct SelectionSet {
            parent_query_node_ix: NodeIndex,
            type_id: CompositeTypeId,
        }
        let mut stack = vec![SelectionSet {
            parent_query_node_ix: self.query.root_node_ix,
            type_id: self.operation.root_object_id.into(),
        }];
        while let Some(SelectionSet {
            parent_query_node_ix,
            type_id,
        }) = stack.pop()
        {
            let ty = type_id.walk(self.schema);
            let mut has_typename = false;
            for edge in self.query.graph.edges(parent_query_node_ix) {
                let SpaceNode::QueryField(QueryFieldNode { id, .. }) = self.query.graph[edge.target()] else {
                    continue;
                };
                match self.query[id].definition_id {
                    Some(definition_id) => {
                        if let Some(type_id) = definition_id.walk(self.schema).ty().definition_id.as_composite_type() {
                            stack.push(SelectionSet {
                                parent_query_node_ix: edge.target(),
                                type_id,
                            });
                            break;
                        }
                    }
                    None => has_typename = true,
                }
            }
            if ty.has_inaccessible_possible_type() && !has_typename {
                self.query.fields.push(QueryField {
                    query_position: None,
                    type_conditions: Default::default(),
                    response_key: None,
                    subgraph_key: None,
                    definition_id: None,
                    argument_ids: Default::default(),
                    location: Location::new(0, 0),
                    flat_directive_id: None,
                });
                let query_field_id = (self.query.fields.len() - 1).into();
                let query_field_node_ix = self.push_query_field_node(query_field_id, FieldFlags::INDISPENSABLE);
                self.query
                    .graph
                    .add_edge(parent_query_node_ix, query_field_node_ix, SpaceEdge::TypenameField);
            }
        }

        Ok(())
    }
}
