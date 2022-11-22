use dynaql::Schema;
use serde_json as _;

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
