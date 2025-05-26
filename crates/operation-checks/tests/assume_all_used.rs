use operation_checks::{AssumeAllUsed, CheckParams, Schema, Severity, check};

#[cfg(test)]
mod tests {
    use super::*;

    use operation_checks::{FieldUsage, check_assuming_all_used};

    #[test]
    fn test_check_assuming_all_used() {
        let source_schema = r#"
            type Query {
                user(id: ID!): User
            }
            
            type User {
                id: ID!
                name: String!
                email: String
            }
        "#;

        let target_schema = r#"
            type Query {
                user(id: ID!, name: String!): User
            }
            
            type User {
                id: ID!
                name: String!
                # email field removed
            }
        "#;

        let source: Schema = async_graphql_parser::parse_schema(source_schema).unwrap().into();
        let target: Schema = async_graphql_parser::parse_schema(target_schema).unwrap().into();
        let diff = graphql_schema_diff::diff(source_schema, target_schema).unwrap();

        let diagnostics = check_assuming_all_used(&source, &target, &diff);

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].message.contains("Query.user.name"));
        assert!(matches!(diagnostics[0].severity, Severity::Error));

        assert!(diagnostics[1].message.contains("User.email"));
        assert!(matches!(diagnostics[1].severity, Severity::Error));
    }

    #[test]
    fn test_trait_usage_vs_concrete_usage() {
        let source_schema = r#"
            type Query {
                user(id: ID!): User
            }
            
            type User {
                id: ID!
                name: String!
                email: String
            }
        "#;

        let target_schema = r#"
            type Query {
                user(id: ID!): User
            }
            
            type User {
                id: ID!
                name: String!
                # email field removed
            }
        "#;

        let source: Schema = async_graphql_parser::parse_schema(source_schema).unwrap().into();
        let target: Schema = async_graphql_parser::parse_schema(target_schema).unwrap().into();
        let diff = graphql_schema_diff::diff(source_schema, target_schema).unwrap();

        let assume_all_used = AssumeAllUsed;
        let params_assume_all = CheckParams {
            source: &source,
            target: &target,
            diff: &diff,
            field_usage: &assume_all_used,
        };
        let diagnostics_assume_all = check(&params_assume_all);

        // Test with empty FieldUsage (no actual usage)
        let empty_usage = FieldUsage::default();
        let params_empty = CheckParams {
            source: &source,
            target: &target,
            diff: &diff,
            field_usage: &empty_usage,
        };
        let diagnostics_empty = check(&params_empty);

        // AssumeAllUsed should find the breaking change
        assert_eq!(diagnostics_assume_all.len(), 1);
        assert!(diagnostics_assume_all[0].message.contains("User.email"));

        // Empty usage should find no breaking changes (since no fields are "used")
        assert_eq!(diagnostics_empty.len(), 0);
    }

    #[test]
    fn test_input_types() {
        let source_schema = r#"
            type Query {
                createUser(input: UserInput!): User
            }
            
            type User {
                id: ID!
                name: String!
            }

            input UserInput {
                name: String!
                email: String
            }
        "#;

        let target_schema = r#"
            type Query {
                createUser(input: UserInput!): User
            }
            
            type User {
                id: ID!
                name: String!
            }

            input UserInput {
                name: String!
                email: String!  # became required
                age: Int!       # new required field
            }
        "#;

        let source: Schema = async_graphql_parser::parse_schema(source_schema).unwrap().into();
        let target: Schema = async_graphql_parser::parse_schema(target_schema).unwrap().into();
        let diff = graphql_schema_diff::diff(source_schema, target_schema).unwrap();

        let diagnostics_assume_all = check_assuming_all_used(&source, &target, &diff);
        assert_eq!(diagnostics_assume_all.len(), 2);
        assert!(
            diagnostics_assume_all
                .iter()
                .any(|d| d.message.contains("UserInput.email") && d.message.contains("became required"))
        );
        assert!(
            diagnostics_assume_all
                .iter()
                .any(|d| d.message.contains("UserInput.age") && d.message.contains("new required field"))
        );

        let empty_usage = FieldUsage::default();
        let params_empty = CheckParams {
            source: &source,
            target: &target,
            diff: &diff,
            field_usage: &empty_usage,
        };
        let diagnostics_empty = check(&params_empty);
        assert_eq!(diagnostics_empty.len(), 0);
    }
}
