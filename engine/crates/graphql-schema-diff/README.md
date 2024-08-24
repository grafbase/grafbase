# graphql-schema-diff

[![crates.io](https://img.shields.io/crates/v/graphql-schema-diff)](https://crates.io/crates/graphql-schema-diff)]
[![docs.rs](https://img.shields.io/docsrs/graphql-schema-diff)](https://docs.rs/graphql-schema-diff/)

This crate implements diffing of two GraphQL schemas, returning a list of changes. It powers the changelog feature and operation checks at Grafbase.

## Example

```rust
use graphql_schema_diff::{diff, Change, ChangeKind, Span};

let source = r#"
  type Pizza {
    id: ID!
    name: String!
    toppings: [Topping!]!
  }

  enum Topping {
    OLIVES
    MUSHROOMS
    PINEAPPLE
  }
"#;

let target = r#"
  type Pizza {
    id: ID!
    name: PizzaName
    toppings: [Topping!]!
  }

  type PizzaName {
    english: String
    italian: String!
  }

  enum Topping {
    OLIVES
    MUSHROOMS
    POTATO
  }
"#;

let changes = diff(source, target).unwrap();

assert_eq!(changes,
   &[
        Change {
            path: String::from("Pizza.name"),
            kind: ChangeKind::ChangeFieldType,
            span: Span::new(38, 47),
        },
        Change {
            path: String::from("PizzaName"),
            kind: ChangeKind::AddObjectType,
            span: Span::new(81, 142),
        },
        Change {
            path: String::from("Topping.PINEAPPLE"),
            kind: ChangeKind::RemoveEnumValue,
            span: Span::new(0, 0),
        },
        Change {
            path: String::from("Topping.POTATO"),
            kind: ChangeKind::AddEnumValue,
            span: Span::new(190, 199),
        }
]);

```

## Cargo features

- `serde`: `Serialize` and `Deserialize` impls for `Change` (default: on).
