mod deny_all;
mod deny_some;
mod error_propagation;
mod error_response;
mod grant_all;
mod types;

use std::sync::Arc;

use extension_catalog::Id;
use integration_tests::federation::{TestExtension, TestExtensionBuilder, TestExtensionConfig};

struct SimpleAuthExt<T> {
    instance: Arc<dyn TestExtension>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: TestExtension> SimpleAuthExt<T> {
    pub fn new(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: TestExtension> TestExtensionBuilder for SimpleAuthExt<T> {
    fn id(&self) -> Id {
        Id {
            name: "simple-auth".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn config(&self) -> TestExtensionConfig {
        TestExtensionConfig {
            kind: extension_catalog::Kind::Authorization(extension_catalog::AuthorizationKind { directives: None }),
            sdl: Some(
                r#"
                directive @auth on FIELD_DEFINITION | OBJECT | INTERFACE
                "#,
            ),
        }
    }

    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        self.instance.clone()
    }
}

fn user() -> serde_json::Value {
    /*
    type Query {
        user: User
    }

    type User {
        id: ID!
        name: String!
        age: Int!
        address: Address
        friends: [User!]
        pets: [Pet!]!
    }

    type Address {
        street: String!
        city: String!
        country: String!
    }

    union Pet = Dog | Cat

    type Dog {
        id: ID!
        name: String!
    }

    type Cat {
        id: ID!
        name: String!
    }
    */
    serde_json::json!({
        "id": "1",
        "name": "Peter",
        "age": 3,
        "address": {"street": "123 Main St", "city": "Springfield", "country": "USA"},
        "friends": [
            {"id": "2", "name": "Alice", "age": 3},
            {"id": "3", "name": "Bob", "age": 4}
        ],
        "pets": [
            { "__typename": "Dog", "id": "1", "name": "Fido" },
            { "__typename": "Cat", "id": "2", "name": "Whiskers" },
        ],
    })
}
