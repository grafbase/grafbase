use dynaql::Schema;
use serde_json as _;

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

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
}

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
}

#[test]
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
        }
        "#,
    )
    .unwrap();

    let sdl = Schema::new(result).sdl();

    insta::assert_snapshot!(sdl);
}

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
}

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
}

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
}

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
}

#[test]
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

    let reg_string = serde_json::to_value(&result).unwrap();
    let sdl = Schema::new(result).sdl();

    insta::assert_json_snapshot!(reg_string);
    insta::assert_snapshot!(sdl);
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
    insta::assert_json_snapshot!(req_string_a);
    insta::assert_snapshot!(sdl_a);

    insta::assert_json_snapshot!(req_string_b);
    insta::assert_snapshot!(sdl_b);

    // Actual test, ensuring they're equivalent.
    assert_eq!(req_string_a, req_string_b);
    assert_eq!(sdl_a, sdl_b);
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
