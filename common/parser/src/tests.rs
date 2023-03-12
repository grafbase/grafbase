use dynaql::registry::{MetaType, Registry};
use dynaql::Schema;
use function_name::named;
use serde_json as _;
use std::collections::{HashMap, HashSet};

use crate::models::from_meta_type;
use crate::rules::visitor::RuleError;

macro_rules! assert_validation_error {
    ($schema:literal, $expected_message:literal) => {
        assert_matches!(
            super::to_registry($schema)
                .err()
                .and_then(super::Error::validation_errors)
                // We don't care whether there are more errors or not.
                // It only matters that we find the expected error.
                .and_then(|errors| errors.into_iter().next()),
            Some(RuleError { message, .. }) if message == $expected_message
        );
    };
}

fn assert_registry_schema_generation(registry: &Registry) {
    for (name, ty) in &registry.types {
        if name.ends_with("Input")
            || name.starts_with("__")
            || ["PageInfo", "Query", "Mutation"].contains(&name.as_str())
        {
            continue;
        }
        if let ty @ MetaType::Object { .. } = ty {
            let schema_opt = from_meta_type(registry, ty);
            // To print the name if there is an issue.
            dbg!(name);
            assert!(schema_opt.is_ok());
        }
    }
}

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
    let result = super::to_registry(
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

    assert_registry_schema_generation(&result);
    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo() {
    let result = super::to_registry(
        r#"
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
        "#,
    )
    .unwrap();

    assert_registry_schema_generation(&result);
    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo_from_template() {
    let result = super::to_registry(
        r#"
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
        "#,
    )
    .unwrap();

    assert_registry_schema_generation(&result);

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo_with_vec() {
    let result = super::to_registry(
        r#"
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
        "#,
    )
    .unwrap();

    assert_registry_schema_generation(&result);

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_todo_with_enum() {
    let result = super::to_registry(
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

    assert_registry_schema_generation(&result);

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_simple_post_with_relation() {
    let result = super::to_registry(
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

    assert_registry_schema_generation(&result);

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_multiple_relations() {
    let result = super::to_registry(
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

    assert_registry_schema_generation(&result);

    assert_snapshot(function_name!(), result);
}

#[test]
#[named]
fn test_many_to_many() {
    let result = super::to_registry(
        r#"
        type User @model {
            name: String!
            organizations: [Organization!]
        }

        type Organization @model {
          name: String!
          users: [User!]
        }
        "#,
    )
    .unwrap();

    assert_registry_schema_generation(&result);

    assert_snapshot(function_name!(), result);
}

#[test]
fn should_ensure_lowercase() {
    let result = super::to_registry(
        r#"
        type Blog @model {
          id: ID!
          truc_break: String! @unique
        }
        "#,
    );

    assert!(result.is_err(), "Should error here");
}

#[test]
#[named]
fn test_model_reserved_fields() {
    let with_reserved_fields = super::to_registry(
        r#"
        type Product @model {
            title: String
            id: ID!
            createdAt: DateTime!
            updatedAt: DateTime!
        }
        "#,
    )
    .unwrap();

    let without_reserved_fields = super::to_registry(
        r#"
        type Product @model {
            title: String
        }
        "#,
    )
    .unwrap();

    let req_string_a = serde_json::to_value(&with_reserved_fields).unwrap();
    let sdl_a = Schema::new(with_reserved_fields).sdl();

    let req_string_b = serde_json::to_value(&without_reserved_fields).unwrap();
    let sdl_b = Schema::new(without_reserved_fields).sdl();

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
    let result = super::to_registry(
        r#"
        type Product @model {
            id: ID!
            content: String!
            rel: Product
        }

        enum Product {
          PRODUCT_A
          PRODUCT_B
        }
        "#,
    );

    assert!(result.is_err(), "Should error here");
}

#[test]
fn should_ensure_reserved_fields_have_correct_type_if_present() {
    assert_validation_error!(
        r#"
        type Product @model {
            id: Int!
        }
        "#,
        "Field 'id' of 'Product' is reserved by @model directive. It must have the type 'ID!' if present."
    );

    assert_validation_error!(
        r#"
        type Dummy @model {
            createdAt: DateTime
        }
        "#,
        "Field 'createdAt' of 'Dummy' is reserved by @model directive. It must have the type 'DateTime!' if present."
    );

    assert_validation_error!(
        r#"
        type Product @model {
            updatedAt: String!
        }
        "#,
        "Field 'updatedAt' of 'Product' is reserved by @model directive. It must have the type 'DateTime!' if present."
    );

    assert!(
        super::to_registry(
            r#"
            type Product @model {
                id: ID!
                createdAt: DateTime!
                updatedAt: DateTime!
            }
            "#
        )
        .is_ok(),
        "Should support specifying explicitly reserved fields."
    );

    assert!(
        super::to_registry(
            r#"
            type Product @model {
                name: String
            }
            "#
        )
        .is_ok(),
        "@model directive should not require reserved fields."
    );

    assert!(
        super::to_registry(
            r#"
            type Product {
                id: Int
                createdAt: DateTime
                updatedAt: String!
            }
            "#
        )
        .is_ok(),
        "Reserved fields only apply with the @model directive."
    );
}

#[test]
fn should_have_unique_fields() {
    assert_validation_error!(
        r#"
        type Product {
            count: Int
            count: Int
        }
        "#,
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

    let result = super::to_registry_with_variables(SCHEMA, &HashMap::new()).expect("must succeed");

    assert_eq!(
        result.required_resolvers,
        HashSet::from(["user/days-inactive".to_owned(), "text/summary".to_owned()])
    );
}

#[test]
#[named]
fn should_support_search_directive() {
    let simple = super::to_registry(
        r#"
            type Product @model {
                title: String @search
            }
            "#,
    );
    assert!(simple.is_ok(), "Search should be supported on @model type");
    let simple = simple.unwrap();
    assert_registry_schema_generation(&simple);
    assert_snapshot(&format!("{}-simple", function_name!()), simple);

    let complex = super::to_registry(
        r#"
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
            "#,
    );

    assert!(complex.is_ok(), "Search should support various field types.");
    let complex = complex.unwrap();
    assert_registry_schema_generation(&complex);
    assert_snapshot(&format!("{}-complex", function_name!()), complex);

    assert_validation_error!(
        r#"
            type Product {
                title: String @search
            }
        "#,
        "The @search directive can only be used on @model types."
    );

    assert_validation_error!(
        r#"
            type Product @model {
                title: JSON @search
            }
        "#,
        "The @search directive cannot be used with the JSON type."
    );
}
