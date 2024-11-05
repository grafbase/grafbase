mod cycle;
mod entities;
mod introspection;
mod mutation;
mod provides;
mod schema1;
mod schema2;
mod shared_root;
mod tea_shop;
mod typename;

use std::{borrow::Cow, num::NonZero};

use engine_parser::{
    types::{DocumentOperations, OperationType, SelectionSet},
    Positioned,
};
use itertools::Itertools;
use schema::{Definition, FieldDefinitionId, ObjectDefinition, ObjectDefinitionId, Schema, Version};
use walker::Walk;

#[ctor::ctor]
fn setup_logging() {
    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .parse(std::env::var("RUST_LOG").unwrap_or("engine_v2_query_planning=debug".to_string()))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .without_time()
        .init();
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct FieldId(NonZero<u16>);

#[derive(Debug, id_derives::IndexedFields)]
struct TestOperation {
    root_object_id: ObjectDefinitionId,
    // Query
    root_selection_set: Vec<FieldId>,
    #[indexed_by(FieldId)]
    fields: Vec<Field>,
}

#[derive(Debug)]
struct Field {
    name: String,
    definition_id: Option<FieldDefinitionId>,
    subselection: Vec<FieldId>,
}

impl TestOperation {
    fn bind(schema: &Schema, query: &str) -> Self {
        let query = engine_parser::parse_query(query).unwrap();
        let DocumentOperations::Single(Positioned {
            node: parsed_operation, ..
        }) = &query.operations
        else {
            unreachable!()
        };
        let root_object_id = match parsed_operation.ty {
            OperationType::Query => schema.graph.root_operation_types_record.query_id,
            OperationType::Mutation => schema.graph.root_operation_types_record.mutation_id.unwrap(),
            OperationType::Subscription => schema.graph.root_operation_types_record.subscription_id.unwrap(),
        };
        let mut operation = TestOperation {
            root_object_id,
            root_selection_set: Vec::new(),
            fields: Vec::new(),
        };
        operation.root_selection_set =
            operation.bind_selection_set(root_object_id.walk(schema), &parsed_operation.selection_set);
        operation
    }

    fn bind_selection_set(&mut self, parent: ObjectDefinition<'_>, selection_set: &SelectionSet) -> Vec<FieldId> {
        let mut field_ids = Vec::new();
        for Positioned { node: selection, .. } in &selection_set.items {
            let field = selection.as_field().unwrap();
            if let Some(definition) = parent.fields().find(|def| def.name() == field.name.node.as_str()) {
                let subselection = match definition.ty().definition() {
                    Definition::Object(obj) => self.bind_selection_set(obj, &field.selection_set.node),
                    _ => Vec::new(),
                };

                self.fields.push(Field {
                    name: field.name.node.to_string(),
                    definition_id: Some(definition.id),
                    subselection,
                });
            } else {
                self.fields.push(Field {
                    name: field.name.node.to_string(),
                    definition_id: None,
                    subselection: Vec::new(),
                });
            }
            let field_id = (self.fields.len() - 1).into();
            field_ids.push(field_id);
        }
        field_ids
    }
}

impl crate::Operation for &mut TestOperation {
    type FieldId = FieldId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static {
        (0..self.fields.len()).map(FieldId::from)
    }

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self.root_selection_set.iter().copied()
    }

    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self[field_id].subselection.iter().copied()
    }

    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str> {
        Cow::Borrowed(&self[field_id].name)
    }

    fn field_definition(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId> {
        self[field_id].definition_id
    }

    fn field_is_equivalent_to(&self, field_id: Self::FieldId, requirement: schema::SchemaField<'_>) -> bool {
        self[field_id].definition_id == Some(requirement.definition_id)
    }

    fn create_potential_extra_field(
        &mut self,
        _petitioner_field_id: Self::FieldId,
        requirement: schema::SchemaField<'_>,
    ) -> Self::FieldId {
        self.fields.push(Field {
            name: requirement.definition().name().to_string(),
            definition_id: Some(requirement.definition_id),
            subselection: Vec::new(),
        });
        (self.fields.len() - 1).into()
    }

    fn finalize_selection_set_extra_fields(&mut self, extra: &[Self::FieldId], _existing: &[Self::FieldId]) {
        for field_id in extra {
            self[*field_id].name.push('*');
        }
    }

    fn field_query_position(&self, field_id: Self::FieldId) -> usize {
        usize::from(field_id)
    }

    fn root_object_id(&self) -> schema::ObjectDefinitionId {
        self.root_object_id
    }
}

#[macro_export]
macro_rules! assert_solving_snapshots {
    ($name: expr, $schema: expr, $query: expr) => {
        let schema = $crate::tests::read_schema($schema);
        let name = $name;
        let query = $query;
        let mut operation = $crate::tests::TestOperation::bind(&schema, query);

        let operation_graph = $crate::operation::OperationGraph::new(&schema, &mut operation).unwrap();
        insta::assert_snapshot!(
            format!("{name}-graph"),
            operation_graph.to_dot_graph(),
            &operation_graph.to_pretty_dot_graph()
        );

        let mut solver = $crate::solve::Solver::initialize(&operation_graph).unwrap();
        insta::assert_snapshot!(
            format!("{name}-solver"),
            solver.to_dot_graph(),
            &solver.to_pretty_dot_graph()
        );

        solver.execute().unwrap();
        insta::assert_snapshot!(
            format!("{name}-solved"),
            solver.to_dot_graph(),
            &solver.to_pretty_dot_graph()
        );

        let solution = solver.into_solution();
        let solution_graph = $crate::Solution::build_partial(operation_graph, solution).unwrap();
        insta::assert_snapshot!(
            format!("{name}-partial-solution"),
            solution_graph.to_dot_graph(),
            &solution_graph.to_pretty_dot_graph()
        );

        let solution_graph = solution_graph.finalize();
        insta::assert_snapshot!(
            format!("{name}-finalized-solution"),
            solution_graph.to_dot_graph(),
            &solution_graph.to_pretty_dot_graph()
        );
    };
}

#[track_caller]
fn read_schema(sdl: &str) -> Schema {
    let graph = federated_graph::from_sdl(sdl).unwrap();
    let config = config::Config::from_graph(graph);
    Schema::build(config, Version::from(Vec::new())).unwrap()
}

#[allow(unused)]
fn strdiff(before: &str, after: &str) -> String {
    similar::TextDiff::from_lines(before, after)
        .iter_all_changes()
        .filter_map(|change| match change.tag() {
            similar::ChangeTag::Equal => None,
            similar::ChangeTag::Delete => Some(('-', change)),
            similar::ChangeTag::Insert => Some(('+', change)),
        })
        .format_with("", |(tag, change), f| f(&format_args!("{}{}", tag, change)))
        .to_string()
}
