mod async_graphql;

use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectionId(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(usize);

/// Usage count of a field in a query.
#[derive(Debug)]
pub struct FieldUsage<'a> {
    pub increment: u64,
    /// field id -> usage count
    pub count_per_field: &'a mut HashMap<FieldId, u64>,
}

impl FieldUsage<'_> {
    pub fn with_increment(self, new_increment: u64) -> Self {
        Self {
            increment: new_increment,
            count_per_field: self.count_per_field,
        }
    }

    /// Register a field usage.
    fn register(&mut self, field_id: FieldId) {
        if let Some(count) = self.count_per_field.get_mut(&field_id) {
            *count += self.increment;
        } else {
            self.count_per_field.insert(field_id, self.increment);
        }
    }
}

/// (type name, field name) -> field type
#[derive(Debug)]
pub struct Schema {
    pub fields: Vec<SchemaField>,
    pub query_type_name: String,
    pub mutation_type_name: String,
    pub subscription_type_name: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaField {
    type_name: String,
    field_name: String,
    /// The type fo the field without any wrapping type (! and []).
    base_type: String,
}

impl Schema {
    fn find_field(&self, type_name: &str, field_name: &str) -> Option<FieldId> {
        self.fields
            .binary_search_by_key(
                &(type_name, field_name),
                |SchemaField {
                     type_name, field_name, ..
                 }| { (type_name, field_name) },
            )
            .map(FieldId)
            .ok()
    }
}

impl std::ops::Index<FieldId> for Schema {
    type Output = SchemaField;

    fn index(&self, index: FieldId) -> &Self::Output {
        &self.fields[index.0]
    }
}

#[derive(Debug)]
pub struct Query {
    /// fragment name -> fragment
    pub fragments: HashMap<String, Fragment>,

    pub operation_type: OperationType,
    pub root_selection: SelectionId,

    /// (parent selection, selection)
    pub selections: Vec<(SelectionId, Selection)>,
}

#[derive(Debug)]
pub struct Fragment {
    pub type_condition: String,
    pub selection: SelectionId,
}

#[derive(Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug)]
pub enum Selection {
    Field {
        field_name: String,
        subselection: Option<SelectionId>,
    },
    FragmentSpread {
        fragment_name: String,
    },
    InlineFragment {
        on: Option<String>,
        selection: SelectionId,
    },
}

/// Given a GraphQL query and the corresponding schema, count the number of times each schema field is used.
pub fn aggregate_field_usage(query: &Query, schema: &Schema, usage: &mut FieldUsage<'_>) {
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
    query: &Query,
    schema: &Schema,
    usage: &mut FieldUsage<'_>,
) {
    let start = query.selections.partition_point(|(id, _)| *id < selection_id);
    let selection_set = query.selections[start..]
        .iter()
        .take_while(|(id, _)| *id == selection_id);

    for (_, selection) in selection_set {
        match selection {
            Selection::Field {
                field_name,
                subselection,
            } => {
                let Some(field_id) = schema.find_field(parent_type_name, field_name) else {
                    continue;
                };

                usage.register(field_id);

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
                let parent_type = on.as_deref().unwrap_or(parent_type_name);
                aggregate_field_usage_inner(*selection, parent_type, query, schema, usage);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_schema(schema: &str) -> Schema {
        async_graphql_parser::parse_schema(schema).unwrap().into()
    }

    fn parse_query(query: &str) -> Query {
        async_graphql_parser::parse_query(query).unwrap().into()
    }

    fn run_test(query: &str, schema: &str, expected: expect_test::Expect) {
        let query = parse_query(query);
        let schema = parse_schema(schema);
        let mut counts = HashMap::new();
        let mut usage = FieldUsage {
            increment: 1,
            count_per_field: &mut counts,
        };

        aggregate_field_usage(&query, &schema, &mut usage);

        let mut counts = counts
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
        let mut counts = HashMap::new();
        let mut usage = FieldUsage {
            increment: 9000,
            count_per_field: &mut counts,
        };

        aggregate_field_usage(&query, &schema, &mut usage);

        let mut counts = counts
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
