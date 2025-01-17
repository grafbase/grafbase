use walker::Walk;

use crate::{NodeFlags, SpaceEdge, SpaceNode};

use super::QuerySolutionSpaceBuilder;

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn add_any_necessary_typename_fields(&mut self) -> crate::Result<()> {
        for selection_set in &mut self.query.selection_sets {
            let ty = selection_set.output_type_id.walk(self.schema);
            if ty.has_inaccessible_possible_type() && selection_set.typename_node_ix_and_petitioner_location.is_none() {
                let ix = self.query.graph.add_node(SpaceNode::Typename {
                    flags: NodeFlags::INDISPENSABLE,
                });
                self.query
                    .graph
                    .add_edge(selection_set.parent_node_ix, ix, SpaceEdge::TypenameField);
            }
        }

        Ok(())
    }
}
