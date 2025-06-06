use crate::{operation::*, schema};
use std::collections::HashMap;

/// Trait for providing usage information about fields, arguments, enum values, and type conditions.
pub trait UsageProvider {
    /// Check if a field is used.
    fn field_is_used(&self, field_id: schema::FieldId) -> bool;

    /// Check if an argument is used.
    fn argument_is_used(&self, argument_id: schema::ArgumentId) -> bool;

    /// Check if an enum value is used.
    fn enum_value_is_used(&self, enum_and_value: &str) -> bool;

    /// Check if an argument with a default was left out.
    fn argument_is_left_out(&self, argument_id: schema::ArgumentId) -> bool;

    /// Check if a type condition is used.
    fn type_condition_is_used(&self, type_condition: &str) -> bool;

    /// Get all used argument IDs for finding used input types.
    fn used_argument_ids(&self) -> Box<dyn Iterator<Item = schema::ArgumentId> + '_>;

    /// Returns true if this provider assumes all input types are used.
    fn assume_all_input_types_used(&self) -> bool {
        false
    }
}

/// Usage count of fields in a set of operations.
#[derive(Debug)]
pub struct FieldUsage {
    pub(crate) increment: u64,
    /// field id -> usage count
    pub(crate) count_per_field: HashMap<schema::FieldId, u64>,
    pub(crate) count_per_field_argument: HashMap<schema::ArgumentId, u64>,
    pub(crate) count_per_enum_value: HashMap<String, u64>,

    /// Arguments that could have been provided but were not. This is fine because they have a
    /// default, but it will be a problem if the default is subsequently removed.
    pub(crate) arguments_with_defaults_left_out_count: HashMap<schema::ArgumentId, u64>,

    /// Usage of interface implementers and union members in type conditions. The key is a string
    /// of the form "parent_type_name.implementer_type_name"
    pub(crate) type_condition_counts: HashMap<String, u64>,
}

impl UsageProvider for FieldUsage {
    fn field_is_used(&self, field_id: schema::FieldId) -> bool {
        self.count_per_field.contains_key(&field_id)
    }

    fn argument_is_used(&self, argument_id: schema::ArgumentId) -> bool {
        self.count_per_field_argument.contains_key(&argument_id)
    }

    fn enum_value_is_used(&self, enum_and_value: &str) -> bool {
        self.count_per_enum_value.contains_key(enum_and_value)
    }

    fn argument_is_left_out(&self, argument_id: schema::ArgumentId) -> bool {
        self.arguments_with_defaults_left_out_count.contains_key(&argument_id)
    }

    fn type_condition_is_used(&self, type_condition: &str) -> bool {
        self.type_condition_counts.contains_key(type_condition)
    }

    fn used_argument_ids(&self) -> Box<dyn Iterator<Item = schema::ArgumentId> + '_> {
        Box::new(self.count_per_field_argument.keys().copied())
    }
}

/// A usage provider that assumes all fields, arguments, and enum values are used.
/// This is useful for checking breaking changes without requiring actual operation data.
#[derive(Debug, Default)]
pub struct AssumeAllUsed;

impl UsageProvider for AssumeAllUsed {
    fn field_is_used(&self, _field_id: schema::FieldId) -> bool {
        true
    }

    fn argument_is_used(&self, _argument_id: schema::ArgumentId) -> bool {
        true
    }

    fn enum_value_is_used(&self, _enum_and_value: &str) -> bool {
        true
    }

    fn argument_is_left_out(&self, _argument_id: schema::ArgumentId) -> bool {
        // When assuming all usage, we also assume arguments with defaults might be left out
        true
    }

    fn type_condition_is_used(&self, _type_condition: &str) -> bool {
        true
    }

    fn used_argument_ids(&self) -> Box<dyn Iterator<Item = schema::ArgumentId> + '_> {
        Box::new(std::iter::empty())
    }

    fn assume_all_input_types_used(&self) -> bool {
        true
    }
}

impl Default for FieldUsage {
    fn default() -> Self {
        FieldUsage {
            increment: 1,
            count_per_field: HashMap::new(),
            count_per_field_argument: HashMap::new(),
            type_condition_counts: HashMap::new(),
            count_per_enum_value: HashMap::new(),
            arguments_with_defaults_left_out_count: HashMap::new(),
        }
    }
}

impl FieldUsage {
    /// Set the increment per field usage. If an operation was used 20 times for example, you
    /// should set this to 20 so each field usage increments by 20.
    pub fn set_increment(&mut self, new_increment: u64) {
        self.increment = new_increment;
    }

    /// Remove all occurrences with fewer than `threshold` requests from the usage counts. This
    /// should only be called after all operations have been aggregated.
    pub fn apply_request_count_threshold(&mut self, threshold: u64) {
        self.count_per_field.retain(|_, count| *count >= threshold);
        self.count_per_field_argument.retain(|_, count| *count >= threshold);
        self.type_condition_counts.retain(|_, count| *count >= threshold);
        self.count_per_enum_value.retain(|_, count| *count >= threshold);
        self.arguments_with_defaults_left_out_count
            .retain(|_, count| *count >= threshold);
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

    fn register_enum_value_usage(&mut self, enum_and_value: String) {
        *self.count_per_enum_value.entry(enum_and_value).or_insert(0) += self.increment;
    }

    fn register_argument_with_default_left_out(&mut self, argument_id: schema::ArgumentId) {
        *self
            .arguments_with_defaults_left_out_count
            .entry(argument_id)
            .or_insert(0) += self.increment;
    }
}

/// Given a GraphQL query and the corresponding schema, count the number of times each schema field is used.
pub fn aggregate_field_usage(query: &Operation, schema: &schema::Schema, usage: &mut FieldUsage) {
    for used_enum_value in &query.enum_values_in_variable_defaults {
        usage.register_enum_value_usage(used_enum_value.clone());
    }

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

                for super::operation::Argument {
                    name,
                    enum_literal_value,
                } in arguments
                {
                    let Some(argument_id) = schema.find_argument((parent_type_name, field_name, name)) else {
                        continue;
                    };

                    usage.register_argument_usage(argument_id);

                    if let Some(enum_value) = enum_literal_value {
                        let enum_type_name = schema[argument_id].base_type.as_str();
                        usage.register_enum_value_usage([enum_type_name, enum_value.as_str()].join("."));
                    }
                }

                if let Some(subselection_id) = subselection {
                    let field_type = &schema[field_id].base_type;
                    aggregate_field_usage_inner(*subselection_id, field_type, query, schema, usage);
                }

                for (argument_id, schema_argument) in schema
                    .iter_field_arguments(field_id)
                    .filter(|(_, schema_arg)| arguments.iter().all(|arg| arg.name != schema_arg.argument_name))
                {
                    if schema_argument.is_required() && schema_argument.has_default {
                        usage.register_argument_with_default_left_out(argument_id);
                    }
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::schema::{ArgumentId, FieldId};

    #[test]
    fn apply_request_count_threshold() {
        let mut usage = FieldUsage {
            increment: 100,
            count_per_field: vec![
                (FieldId(1), 100),
                (FieldId(2), 200),
                (FieldId(3), 300),
                (FieldId(4), 400),
                (FieldId(5), 500),
            ]
            .into_iter()
            .collect(),
            count_per_field_argument: vec![
                (ArgumentId(1), 100),
                (ArgumentId(2), 200),
                (ArgumentId(3), 300),
                (ArgumentId(4), 400),
                (ArgumentId(5), 500),
            ]
            .into_iter()
            .collect(),
            type_condition_counts: vec![
                ("E.F".to_string(), 300),
                ("G.H".to_string(), 400),
                ("C.D".to_string(), 200),
                ("I.J".to_string(), 500),
                ("A.B".to_string(), 100),
            ]
            .into_iter()
            .collect(),
            count_per_enum_value: [("Color.RED".to_string(), 200), ("Animal.GIRAFFE".to_string(), 300)]
                .into_iter()
                .collect(),
            arguments_with_defaults_left_out_count: [
                (ArgumentId(1), 100),
                (ArgumentId(1000), 1),
                (ArgumentId(100), 1000),
            ]
            .into_iter()
            .collect(),
        };

        usage.apply_request_count_threshold(300);

        fn keys<K: Ord, V>(hm: &HashMap<K, V>) -> Vec<&K> {
            let mut out: Vec<_> = hm.keys().collect();
            out.sort();
            out
        }

        assert_eq!(keys(&usage.count_per_field), &[&FieldId(3), &FieldId(4), &FieldId(5)]);
        assert_eq!(
            keys(&usage.count_per_field_argument),
            &[&ArgumentId(3), &ArgumentId(4), &ArgumentId(5)]
        );

        assert_eq!(keys(&usage.type_condition_counts), &["E.F", "G.H", "I.J"]);
        assert_eq!(keys(&usage.count_per_enum_value), &["Animal.GIRAFFE"]);
        assert_eq!(keys(&usage.arguments_with_defaults_left_out_count), &[&ArgumentId(100)]);
    }
}
