use crate::{operation::*, schema};
use std::collections::HashMap;

/// Usage count of fields in a set of operations.
#[derive(Debug)]
pub struct FieldUsage {
    pub(crate) increment: u64,
    /// field id -> usage count
    pub(crate) count_per_field: HashMap<schema::FieldId, u64>,
    pub(crate) count_per_field_argument: HashMap<schema::ArgumentId, u64>,

    /// Usage of interface implementers and union members in type conditions. The key is a string
    /// of the form "parent_type_name.implementer_type_name"
    pub(crate) type_condition_counts: HashMap<String, u64>,
}

impl Default for FieldUsage {
    fn default() -> Self {
        FieldUsage {
            increment: 1,
            count_per_field: HashMap::new(),
            count_per_field_argument: HashMap::new(),
            type_condition_counts: HashMap::new(),
        }
    }
}

impl FieldUsage {
    /// Set the increment per field usage. If an operation was used 20 times for example, you
    /// should set this to 20 so each field usage increments by 20.
    pub fn set_increment(&mut self, new_increment: u64) {
        self.increment = new_increment;
    }

    /// Register a field usage.
    fn register_field_usage(&mut self, field_id: schema::FieldId) {
        if let Some(count) = self.count_per_field.get_mut(&field_id) {
            *count += self.increment;
        } else {
            self.count_per_field.insert(field_id, self.increment);
        }
    }

    fn register_argument_usage(&mut self, argument_id: schema::ArgumentId) {
        if let Some(count) = self.count_per_field_argument.get_mut(&argument_id) {
            *count += self.increment;
        } else {
            self.count_per_field_argument.insert(argument_id, self.increment);
        }
    }
}

/// Given a GraphQL query and the corresponding schema, count the number of times each schema field is used.
pub fn aggregate_field_usage(query: &Operation, schema: &schema::Schema, usage: &mut FieldUsage) {
    let ty = match query.operation_type {
        OperationType::Query => &schema.query_type_name,
        OperationType::Mutation => &schema.mutation_type_name,
        OperationType::Subscription => &schema.subscription_type_name,
    };
    aggregate_field_usage_inner(query.root_selection, ty, query, schema, usage);
}

fn aggregate_field_usage_inner(
    selection_id: SelectionId,
    parent_type_name: &str,
    query: &Operation,
    schema: &schema::Schema,
    usage: &mut FieldUsage,
) {
    let start = query.selections.partition_point(|(id, _)| *id < selection_id);
    let selection_set = query.selections[start..]
        .iter()
        .take_while(|(id, _)| *id == selection_id);

    for (_, selection) in selection_set {
        match selection {
            Selection::Field {
                field_name,
                arguments,
                subselection,
            } => {
                let Some(field_id) = schema.find_field(parent_type_name, field_name) else {
                    continue;
                };

                usage.register_field_usage(field_id);

                for arg_name in arguments {
                    let Some(argument_id) = schema.find_argument((parent_type_name, field_name, &arg_name)) else {
                        continue;
                    };

                    usage.register_argument_usage(argument_id);
                }

                if let Some(subselection_id) = subselection {
                    let field_type = &schema[field_id].base_type;
                    aggregate_field_usage_inner(*subselection_id, field_type, query, schema, usage);
                }
            }
            Selection::FragmentSpread { fragment_name } => {
                let Some(Fragment {
                    type_condition,
                    selection,
                }) = query.fragments.get(fragment_name)
                else {
                    continue;
                };

                aggregate_field_usage_inner(*selection, type_condition, query, schema, usage);
            }
            Selection::InlineFragment { on, selection } => {
                let subselection_parent_type = if let Some(on) = on {
                    *usage
                        .type_condition_counts
                        .entry([parent_type_name, on].join("."))
                        .or_insert(0) += 1;

                    on
                } else {
                    parent_type_name
                };

                aggregate_field_usage_inner(*selection, subselection_parent_type, query, schema, usage);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_schema(schema: &str) -> schema::Schema {
        async_graphql_parser::parse_schema(schema).unwrap().into()
    }

    fn parse_query(query: &str) -> Operation {
        async_graphql_parser::parse_query(query).unwrap().into()
    }

    fn run_test(query: &str, schema: &str, expected: expect_test::Expect) {
        let query = parse_query(query);
        let schema = parse_schema(schema);
        let mut usage = FieldUsage::default();

        aggregate_field_usage(&query, &schema, &mut usage);

        let mut counts = usage
            .count_per_field
            .into_iter()
            .map(|(id, count)| format!("{}.{} => {count}", schema[id].type_name, schema[id].field_name))
            .collect::<Vec<_>>();

        counts.sort();

        expected.assert_debug_eq(&counts);
    }

    #[test]
    fn basic() {
        let query = r#"
            mutation {
                createTodo {
                    id
                        text
                        completed
                        author { name }
                }
                deleteUser { email name }
            }
        "#;
        let schema = r#"
            type Mutation {
                createTodo: Todo!
                    deleteUser: User
            }

        type Todo {
            id: ID!
                text: String!
                completed: Boolean!
                charactersCount: Int!
                author: User!
        }

        type User {
            name: String
                email: String
        }
        "#;
        let expected = expect_test::expect![[r#"
            [
                "Mutation.createTodo => 1",
                "Mutation.deleteUser => 1",
                "Todo.author => 1",
                "Todo.completed => 1",
                "Todo.id => 1",
                "Todo.text => 1",
                "User.email => 1",
                "User.name => 2",
            ]
        "#]];

        run_test(query, schema, expected);
    }

    #[test]
    fn with_fragment() {
        let query = r#"
            fragment TodoFields on Todo {
                id
                    text
                    completed
                    author { name }
            }

        mutation {
            createTodo {
                ...TodoFields
            }
            deleteUser { email name }
        }
        "#;

        let schema = r#"
            type Mutation {
                createTodo: Todo!
                    deleteUser: User
            }

        type Todo {
            id: ID!
                text: String!
                completed: Boolean!
                charactersCount: Int!
                author: User!
        }

        type User {
            name: String
                email: String
        }
        "#;

        let expected = expect_test::expect![[r#"
            [
                "Mutation.createTodo => 1",
                "Mutation.deleteUser => 1",
                "Todo.author => 1",
                "Todo.completed => 1",
                "Todo.id => 1",
                "Todo.text => 1",
                "User.email => 1",
                "User.name => 2",
            ]
        "#]];

        run_test(query, schema, expected);
    }

    #[test]
    fn with_inline_fragment() {
        let query = r#"
            mutation {
                createTodo {
                    ... on Error {
                        message
                    }
                    ... on Todo {
                        id
                        text
                        completed
                        author { name }
                    }
                }
                deleteUser { email name }
            }
        "#;

        let schema = r#"
            type Mutation {
                createTodo: Todo!
                    deleteUser: User
            }

        union CreateTodoResult = Todo | Error

            type Error {
                message: String!
            }

            type Todo {
                id: ID!
                text: String!
                completed: Boolean!
                charactersCount: Int!
                author: User
            }

            type User {
                name: String
                email: String
            }
        "#;

        let expected = expect_test::expect![[r#"
            [
                "Error.message => 1",
                "Mutation.createTodo => 1",
                "Mutation.deleteUser => 1",
                "Todo.author => 1",
                "Todo.completed => 1",
                "Todo.id => 1",
                "Todo.text => 1",
                "User.email => 1",
                "User.name => 2",
            ]
        "#]];

        run_test(query, schema, expected);
    }

    #[test]
    fn selection_on_field_that_does_not_exist_in_schema() {
        let query = r#"
            mutation {
                createTodo {
                    ... on Error {
                        message
                        code # does not exist
                    }
                    ... on Todo {
                        id
                        text
                        completed
                        author { name }
                    }
                }
            }
        "#;

        let schema = r#"
            type Mutation {
                createTodo: Todo!
            }

            union CreateTodoResult = Todo | Error

            type Error {
                message: String!
            }

            type Todo {
                id: ID!
                text: String!
                completed: Boolean!
                charactersCount: Int!
            }
        "#;

        let expected = expect_test::expect![[r#"
            [
                "Error.message => 1",
                "Mutation.createTodo => 1",
                "Todo.completed => 1",
                "Todo.id => 1",
                "Todo.text => 1",
            ]
        "#]];

        run_test(query, schema, expected);
    }

    #[test]
    fn increment_more_than_1() {
        let query = r#"
            query Test {
                ping { pong }
                bing: ping { bong: pong message }
            }
        "#;

        let schema = r#"
            schema {
                query: MyQuery
            }

            type MyQuery {
                ping: Pong
            }

            interface Pong {
                pong: String!
                message: String!
            }
        "#;

        let query = parse_query(query);
        let schema = parse_schema(schema);

        let mut usage = FieldUsage {
            increment: 9000,
            ..Default::default()
        };

        aggregate_field_usage(&query, &schema, &mut usage);

        let mut counts = usage
            .count_per_field
            .into_iter()
            .map(|(id, count)| format!("{}.{} => {count}", schema[id].type_name, schema[id].field_name))
            .collect::<Vec<_>>();

        counts.sort();

        let expected = expect_test::expect![[r#"
            [
                "MyQuery.ping => 18000",
                "Pong.message => 9000",
                "Pong.pong => 18000",
            ]
        "#]];

        expected.assert_debug_eq(&counts);
    }
}
