use std::collections::{HashMap, HashSet};

use crate::{planning::defers::DeferId, TypeRelationships};

use super::{ConcreteShapeId, FieldRecord, ObjectShapeId, ObjectShapeRecord, TypeConditionId, TypeConditionNode};

#[derive(Default)]
pub struct OutputShapesBuilder {
    pub(super) objects: Vec<ObjectShapeRecord>,
    pub(super) type_conditions: Vec<TypeConditionNode>,
    pub(super) defer_roots: Vec<(ConcreteShapeId, DeferId)>,
}

impl OutputShapesBuilder {
    pub fn insert_concrete_object(&mut self, fields: Vec<FieldRecord>, defers: Vec<DeferId>) -> ConcreteShapeId {
        let object_id = self.insert_record(ObjectShapeRecord::Concrete { fields });
        let concrete_shape_id = ConcreteShapeId(object_id.0);

        self.defer_roots
            .extend(defers.into_iter().map(|defer_id| (concrete_shape_id, defer_id)));

        concrete_shape_id
    }

    pub fn insert_polymorphic_object(
        &mut self,
        fallback_fields: Vec<FieldRecord>,
        fallback_defers: Vec<DeferId>,
        fields_for_typeconditions: Vec<(String, Vec<FieldRecord>, Vec<DeferId>)>,
        type_relationships: &dyn TypeRelationships,
    ) -> ObjectShapeId {
        let fallback = self.insert_concrete_object(fallback_fields, fallback_defers);

        let types = fields_for_typeconditions
            .into_iter()
            .map(|(typename, fields, defers)| (typename, self.insert_concrete_object(fields, defers)))
            .collect::<Vec<_>>();

        let name_indices = types
            .iter()
            .enumerate()
            .map(|(index, (name, _))| (name.as_str(), index))
            .collect::<HashMap<_, _>>();

        // Build an adjacency list of subtype relationships
        let mut roots = vec![];
        let mut subtypes = vec![vec![]; types.len()];
        for (name, subtype_index) in &name_indices {
            let mut supertype_indices = type_relationships
                .supertypes(name)
                .filter_map(|supertype| name_indices.get(supertype))
                .peekable();

            if supertype_indices.peek().is_none() {
                roots.push(subtype_index)
            }

            for supertype_index in supertype_indices {
                subtypes[*supertype_index].push(*subtype_index)
            }
        }

        let mut ids = vec![None; types.len()];

        // Top sort our subtypes so we build dependencies first, and so we can order
        // subtypes by specificity
        let sorted_indexes = topological_sort(&subtypes).expect("TODO: GB-6966");
        let mut sort_order = vec![0; sorted_indexes.len()];
        for (sort_position, index) in sorted_indexes.iter().enumerate() {
            sort_order[*index] = sort_position
        }

        for index in sorted_indexes {
            let (type_condition, concrete_shape) = &types[index];

            let mut subtype_indexes = std::mem::take(&mut subtypes[index]);
            subtype_indexes.sort_by_key(|index| sort_order[*index]);
            let subtypes = self.unwrap_and_box_nodes(subtype_indexes.into_iter().map(|index| ids[index]));

            ids[index] = Some(self.insert_type_tree_node(TypeConditionNode {
                type_condition: type_condition.to_string(),
                concrete_shape: *concrete_shape,
                subtypes,
            }));
        }

        let type_conditions = self.unwrap_and_box_nodes(roots.into_iter().map(|index| ids[*index]));

        self.insert_record(ObjectShapeRecord::Polymorphic {
            type_conditions,
            fallback,
        })
    }

    fn insert_record(&mut self, record: ObjectShapeRecord) -> ObjectShapeId {
        let id = ObjectShapeId(u16::try_from(self.objects.len()).expect("too many objects, what the hell"));
        self.objects.push(record);
        id
    }

    fn insert_type_tree_node(&mut self, node: TypeConditionNode) -> TypeConditionId {
        let id = TypeConditionId(
            u16::try_from(self.type_conditions.len()).expect("too many type tree nodes, what is happening"),
        );
        self.type_conditions.push(node);
        id
    }

    fn unwrap_and_box_nodes(
        &self,
        nodes: impl ExactSizeIterator<Item = Option<TypeConditionId>>,
    ) -> Box<[TypeConditionId]> {
        nodes
            .map(|option| option.expect("the node to be present because we did a topsort"))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

fn topological_sort(adjacency_list: &Vec<Vec<usize>>) -> Result<Vec<usize>, ()> {
    fn visit(
        adjacency_list: &Vec<Vec<usize>>,
        node: usize,
        fresh_nodes: &mut HashSet<usize>,
        nodes_visited_this_traversal: &mut HashSet<usize>,
        output: &mut Vec<usize>,
    ) -> Result<(), ()> {
        if !fresh_nodes.contains(&node) {
            return Ok(());
        }
        if nodes_visited_this_traversal.contains(&node) {
            // This indicates a cycle, which shouldn't be able to happen in a well formed GraphQL schema
            return Err(());
        }

        nodes_visited_this_traversal.insert(node);

        for neighbour in &adjacency_list[node] {
            visit(
                adjacency_list,
                *neighbour,
                fresh_nodes,
                nodes_visited_this_traversal,
                output,
            )?;
        }

        nodes_visited_this_traversal.remove(&node);
        fresh_nodes.remove(&node);
        output.push(node);

        Ok(())
    }

    let mut still_to_visit = adjacency_list
        .iter()
        .enumerate()
        .map(|(i, _)| i)
        .collect::<HashSet<_>>();
    let mut nodes_visited_this_traversal = HashSet::new();
    let mut output = Vec::with_capacity(adjacency_list.len());

    while let Some(node) = still_to_visit.iter().next() {
        visit(
            adjacency_list,
            *node,
            &mut still_to_visit,
            &mut nodes_visited_this_traversal,
            &mut output,
        )?;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_polymorphic_object() {
        let mut builder = OutputShapesBuilder::default();

        let object_id = builder.insert_polymorphic_object(
            vec![leaf("hello")],
            vec![],
            vec![
                ("Node".into(), vec![leaf("id")], vec![]),
                ("NamedNode".into(), vec![leaf("id"), leaf("name")], vec![]),
                ("User".into(), vec![leaf("id"), leaf("name"), leaf("other")], vec![]),
            ],
            &Types,
        );

        let node_type_conditions = builder.objects[object_id.0 as usize].as_type_conditions();

        assert_eq!(node_type_conditions.len(), 1);

        let node = &builder.type_conditions[node_type_conditions[0].0 as usize];
        assert_eq!(node.type_condition, "Node");

        assert_eq!(node.subtypes.len(), 2);

        // The User type needs to be first because it is the most specific.
        let user = &builder.type_conditions[node.subtypes[0].0 as usize];
        assert_eq!(user.type_condition, "User");
        assert!(user.subtypes.is_empty());

        // NamedNode should be next
        let named_node = &builder.type_conditions[node.subtypes[1].0 as usize];
        assert_eq!(named_node.type_condition, "NamedNode");
        assert_eq!(named_node.subtypes.len(), 1);
    }

    impl ObjectShapeRecord {
        fn as_type_conditions(&self) -> &[TypeConditionId] {
            let ObjectShapeRecord::Polymorphic { type_conditions, .. } = self else {
                unreachable!()
            };

            type_conditions
        }
    }

    struct Types;

    impl TypeRelationships for Types {
        fn type_condition_matches(&self, _type_condition: &str, _typename: &str) -> bool {
            unimplemented!("dont need this here")
        }

        fn supertypes<'b>(&'b self, typename: &str) -> Box<dyn Iterator<Item = &str> + 'b> {
            match typename {
                "Node" => Box::new([].into_iter()),
                "NamedNode" => Box::new(["Node"].into_iter()),
                "User" => Box::new(["Node", "NamedNode"].into_iter()),
                _ => unimplemented!(),
            }
        }
    }

    fn leaf(response_key: &str) -> FieldRecord {
        FieldRecord {
            response_key: response_key.into(),
            defer: None,
            subselection_shape: None,
        }
    }
}
