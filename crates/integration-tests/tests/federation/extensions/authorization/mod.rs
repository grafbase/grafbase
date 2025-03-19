mod authenticated;
mod deny_all;
mod deny_some;
mod error_propagation;
mod error_response;
mod grant_all;
mod headers;
mod injection;
mod query;
mod requires_scopes;
mod response;

use std::sync::Arc;

use extension_catalog::Id;
use integration_tests::federation::{TestExtension, TestExtensionBuilder, TestManifest};

pub use headers::*;

pub struct AuthorizationExt<T> {
    instance: Arc<dyn TestExtension>,
    name: &'static str,
    sdl: Option<&'static str>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: TestExtension> AuthorizationExt<T> {
    pub fn new(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            name: "authorization",
            sdl: None,
            phantom: std::marker::PhantomData,
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_sdl(mut self, sdl: &'static str) -> Self {
        self.sdl = Some(sdl);
        self
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }
}

impl<T: TestExtension> TestExtensionBuilder for AuthorizationExt<T> {
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: self.name.to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            kind: extension_catalog::Kind::Authorization(extension_catalog::AuthorizationKind {
                authorization_directives: None,
            }),
            sdl: self.sdl.or(Some(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["FieldSet", "InputValueSet"])

                scalar JSON

                directive @auth(input: JSON, fields: FieldSet, args: InputValueSet) on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
                "#,
            )),
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
