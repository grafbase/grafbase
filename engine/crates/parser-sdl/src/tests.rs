use std::collections::{HashMap, HashSet};

use common_types::UdfKind;
use engine::{registry::Registry, Schema};
use function_name::named;
use serde_json as _;

macro_rules! assert_validation_error {
    ($schema:expr, $expected_message:literal) => {
        assert_matches!(
            $crate::parse_registry($schema)
                .err()
                .and_then(crate::Error::validation_errors)
                // We don't care whether there are more errors or not.
                // It only matters that we find the expected error.
                .and_then(|errors| errors.into_iter().next()),
            Some(crate::RuleError { message, .. }) => {
                assert_eq!(message, $expected_message);
            }
        )
    };
}

pub(crate) use assert_validation_error;

fn assert_snapshot(name: &str, registry: Registry) {
    let _reg_string = serde_json::to_value(&registry).unwrap();
    let sdl = Schema::new(registry).sdl();

    insta::with_settings!({sort_maps => true}, {
        // insta::assert_json_snapshot!(format!("{name}-registry"), reg_string);
        insta::assert_snapshot!(format!("{name}-sdl"), sdl);
    });
}

#[test]
#[named]
fn test_simple_product() {
    let result = super::parse_registry(
        r#"
        type Product @model {
            id: ID!
            name: String!
            """
            The product's price in $
            """
            price: Int!
        }
        "#,
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo() {
    let result = super::parse_registry(
        r"
        type Todo @model {
          id: ID!
          content: String!
          author: Author
        }

        type Author {
          name: String!
          lastname: String!
          pseudo: String
          truc: Truc!
        }

        type Truc {
          name: String!
        }
        ",
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo_from_template() {
    let result = super::parse_registry(
        r"
        type TodoList @model {
          id: ID!
          title: String!
          todos: [Todo]
        }

        type Todo @model {
          id: ID!
          title: String!
          complete: Boolean
          list: TodoList
        }
        ",
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo_with_vec() {
    let result = super::parse_registry(
        r"
        type Todo @model {
          id: ID!
          content: String!
          authors: [Author]
        }

        type Author {
          name: String!
          lastname: String!
          pseudo: String
          truc: Truc!
        }

        type Truc {
          name: String!
        }
        ",
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo_with_enum() {
    let result = super::parse_registry(
        r#"
        """
        A TodoType
        """
        enum TodoType {
          TODO1

          """
          A Type 2 for TODO
          """
          TODO2
        }

        type Todo @model {
          id: ID!
          content: String!
          authors: [Author]
          ty: TodoType!
        }

        type Author {
          name: String!
          lastname: String!
          pseudo: String
          truc: Truc!
        }

        type Truc {
          name: String!
        }
        "#,
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_post_with_relation() {
    let result = super::parse_registry(
        r#"
        enum Country {
          FRANCE
          NOT_FRANCE
        }

        type Blog @model {
          id: ID!
          posts: [Post]
          owner: Author!
        }

        type Post @model {
          id: ID!
          blog: Blog
          content: String!
          authors: [Author] @relation(name: "published")
        }

        type Author @model {
          id: ID!
          name: String!
          lastname: String!
          country: Country!
          posts: [Post] @relation(name: "published")
        }
        "#,
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_multiple_relations() {
    let result = super::parse_registry(
        r#"
        type Author @model {
          id: ID!
          lastname: String!
          published: [Post] @relation(name: "published")
          commented: [Comment] @relation(name: "commented")
        }

        type Post @model {
          id: ID!
          content: String!
          author: Author @relation(name: "published")
          comments: [Comment] @relation(name: "comments")
        }

        type Comment @model {
          id: ID!
          author: Author! @relation(name: "commented")
          post: Post @relation(name: "comments")
          comment: String!
          like: Int!
        }
        "#,
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_many_to_many() {
    let result = super::parse_registry(
        r"
        type User @model {
            name: String!
            organizations: [Organization!]
        }

        type Organization @model {
          name: String!
          users: [User!]
        }
        ",
    )
    .unwrap();

    assert_snapshot(function_name!(), result);
}

#[test]
fn should_ensure_lowercase() {
    let result = super::parse_registry(
        r"
        type Blog @model {
          id: ID!
          truc_break: String! @unique
        }
        ",
    );

    assert!(result.is_err(), "Should error here");
}

#[test]
fn should_forbid_use_of_reserved_fields() {
    assert_validation_error!(
        r"
        type Product @model {
            ALL: Int
        }
        ",
        "Field name 'ALL' is reserved and cannot be used."
    );
    assert_validation_error!(
        r"
        type Product @model {
            ANY: Int
        }
        ",
        "Field name 'ANY' is reserved and cannot be used."
    );
    assert_validation_error!(
        r"
        type Product @model {
            NONE: Int
        }
        ",
        "Field name 'NONE' is reserved and cannot be used."
    );
    assert_validation_error!(
        r"
        type Product @model {
            NOT: Int
        }
        ",
        "Field name 'NOT' is reserved and cannot be used."
    );
}

#[test]
fn test_schema() {
    let nested_schema = super::parse_registry(
        r"
        type User @model {
            id: ID!
            nested: Nested
        }

        type Nested {
          requiredField: String!
        }
        ",
    )
    .unwrap();

    let req_string_a = serde_json::to_value(&nested_schema).unwrap();
    let sdl_a = Schema::new(nested_schema).sdl();

    insta::assert_json_snapshot!(req_string_a);
    insta::assert_snapshot!(sdl_a);
}

#[test]
#[named]
fn test_model_reserved_fields() {
    let with_metadata_fields = super::parse_registry(
        r"
        type Product @model {
            title: String
            id: ID!
            createdAt: DateTime!
            updatedAt: DateTime!
        }
        ",
    )
    .unwrap();

    let without_metadata_fields = super::parse_registry(
        r"
        type Product @model {
            title: String
        }
        ",
    )
    .unwrap();

    let req_string_a = serde_json::to_value(&with_metadata_fields).unwrap();
    let sdl_a = Schema::new(with_metadata_fields).sdl();

    let req_string_b = serde_json::to_value(&without_metadata_fields).unwrap();
    let sdl_b = Schema::new(without_metadata_fields).sdl();

    // Comparing snaphots for better test errors first
    insta::assert_json_snapshot!(format!("{}-req-a", function_name!()), req_string_a);
    insta::assert_snapshot!(format!("{}-sdl-a", function_name!()), sdl_a);

    insta::assert_json_snapshot!(format!("{}-req-b", function_name!()), req_string_b);
    insta::assert_snapshot!(format!("{}-sdl-b", function_name!()), sdl_b);

    // Actual test, ensuring they're equivalent.
    assert_eq!(req_string_a, req_string_b);
    assert_eq!(sdl_a, sdl_b);
}

#[test]
fn should_not_allow_same_enum_and_type() {
    let result = super::parse_registry(
        r"
        type Product @model {
            id: ID!
            content: String!
            rel: Product
        }

        enum Product {
          PRODUCT_A
          PRODUCT_B
        }
        ",
    );

    assert!(result.is_err(), "Should error here");
}

#[test]
fn should_ensure_reserved_fields_have_correct_type_if_present() {
    assert_validation_error!(
        r"
        type Product @model {
            id: Int!
        }
        ",
        "Field 'id' of 'Product' is reserved by @model directive. It must have the type 'ID!' if present."
    );

    assert_validation_error!(
        r"
        type Dummy @model {
            createdAt: DateTime
        }
        ",
        "Field 'createdAt' of 'Dummy' is reserved by @model directive. It must have the type 'DateTime!' if present."
    );

    assert_validation_error!(
        r"
        type Product @model {
            updatedAt: String!
        }
        ",
        "Field 'updatedAt' of 'Product' is reserved by @model directive. It must have the type 'DateTime!' if present."
    );

    assert!(
        super::parse_registry(
            r"
            type Product @model {
                id: ID!
                createdAt: DateTime!
                updatedAt: DateTime!
            }
            "
        )
        .is_ok(),
        "Should support specifying explicitly reserved fields."
    );

    assert!(
        super::parse_registry(
            r"
            type Product @model {
                name: String
            }
            "
        )
        .is_ok(),
        "@model directive should not require reserved fields."
    );

    assert!(
        super::parse_registry(
            r"
            type Product {
                id: Int
                createdAt: DateTime
                updatedAt: String!
            }
            "
        )
        .is_ok(),
        "Reserved fields only apply with the @model directive."
    );
}

#[test]
fn should_have_unique_fields() {
    assert_validation_error!(
        r"
        type Product {
            count: Int
            count: Int
        }
        ",
        "Field 'count' cannot be defined multiple times."
    );
}

#[test]
fn should_validate_relation_name() {
    assert_validation_error!(
        r#"
            type Todo @model {
                secondaryAuthors: [Author] @relation(name: "second-author")
            }

            type Author @model {
                id: ID!
            }
        "#,
        "Relation names should only contain [_a-zA-Z0-9] but second-author does not"
    );
}

#[test]
fn should_pick_up_required_resolvers() {
    let variables = HashMap::new();
    const SCHEMA: &str = r#"
        type User @model {
            name: String!
            email: String!
            lastSignIn: DateTime
            daysInactive: Int! @resolver(name: "user/days-inactive")
        }

        type Post @model {
            author: User!
            contents: String!
            computedSummary: String! @resolver(name: "text/summary")
        }

        type Comment @model {
            author: User!
            post: Post!
            contents: String!
            computedSummary: String! @resolver(name: "text/summary")
        }
    "#;

    let result = super::to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");

    assert_eq!(
        result.required_udfs,
        HashSet::from([
            (UdfKind::Resolver, "user/days-inactive".to_owned()),
            (UdfKind::Resolver, "text/summary".to_owned())
        ])
    );
}

#[test]
#[named]
fn should_support_search_directive() {
    let simple = super::parse_registry(
        r"
            type Product @model {
                title: String @search
            }
            ",
    );
    assert!(simple.is_ok(), "Search should be supported on @model type");
    let simple = simple.unwrap();
    assert_snapshot(&format!("{}-simple", function_name!()), simple);

    let complex = super::parse_registry(
        r"
            type Product @model {
              ip: IPAddress @search
              timestamp: Timestamp! @search
              url: URL @search
              email: [Email] @search
              phone: PhoneNumber @search
              date: [Date!]! @search
              datetime: DateTime @search
              text: [[String]] @search
              int: Int @search
              float: Float @search
              bool: Boolean @search
            }
            ",
    );

    assert!(complex.is_ok(), "Search should support various field types.");
    let complex = complex.unwrap();
    assert_snapshot(&format!("{}-complex", function_name!()), complex);

    let model_directive = super::parse_registry(
        r"
            type Product @model @search {
              ip: IPAddress
              timestamp: Timestamp!
              url: URL
              email: [Email]
              phone: PhoneNumber
              date: [Date!]!
              datetime: DateTime
              text: [[String]]
              int: Int
              float: Float
              bool: Boolean
              json: JSON
            }
            ",
    );

    assert!(
        model_directive.is_ok(),
        "Search should support model @search directive."
    );
    let model_directive = model_directive.unwrap();
    assert_snapshot(&format!("{}-model_directive", function_name!()), model_directive);

    let enum_field = super::parse_registry(
        r"
            enum Status {
                ACTIVE
                INACTIVE
            }

            enum Pet {
                CAT
                DOG
            }

            type Product @model @search {
                status: Status
                pet: Pet!
                pets: [Pet!]
                name: String
            }
            ",
    );

    assert!(enum_field.is_ok(), "Search should support model @search directive.");
    let model_directive = enum_field.unwrap();
    assert_snapshot(&format!("{}-enum-field", function_name!()), model_directive);

    assert_validation_error!(
        r"
            type Product {
                title: String @search
            }
        ",
        "The @search directive can only be used on @model types."
    );

    assert_validation_error!(
        r"
            type Product @search {
                title: String
            }
        ",
        "The @search directive can only be used on @model types."
    );

    assert_validation_error!(
        r"
            type Product @model {
                title: JSON @search
            }
        ",
        "The @search directive cannot be used with the JSON type."
    );
}

#[test]
#[named]
fn test_search_enums_placed_after_use() {
    let registry = super::parse_registry(
        r"
        type User @model @search {
            role: UserRoles! @default(value: EMPLOYEE)
        }

        enum UserRoles {
            EMPLOYEE
            CEO
        }
        ",
    )
    .unwrap();
    assert_snapshot(function_name!(), registry);
}

#[test]
fn test_name_clashes_dont_cause_panic() {
    let schema = r"
        type User {
            id: ID!
        }

        input UserInput {
            id: ID!
        }
    ";
    super::to_parse_result_with_variables(schema, &HashMap::new()).expect("must succeed");
}

#[test]
fn test_with_lowercase_model_names() {
    assert_validation_error!(
        r"
            type user @model {
                name: String!
                projects: [Project!]
            }

            type Project @model {
                createdBy: user!
            }
        ",
        "Models must be named in PascalCase.  Try renaming user to User."
    );
}

#[test]
fn test_missing_model_relation_gb4652() {
    assert_validation_error!(
        r"
        type Space @model {
          posts: [Post]
        }

        type Post {
          space: Space!
        }
        ",
        "Non @model type (Post) cannot have a field (space) with a @model type (Space). Consider adding @model directive to Post."
    );
}

#[test]
fn test_experimental() {
    let result = super::parse_registry(
        r"
            extend schema @experimental(kv: true)
        ",
    )
    .unwrap();

    assert!(result.enable_kv);

    let result = super::parse_registry(
        r"
            extend schema @experimental(kv: false)
        ",
    )
    .unwrap();

    assert!(!result.enable_kv);
}
