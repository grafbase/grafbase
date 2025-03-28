mod authenticated;
mod backwards_compatibility;
mod deny_all;
mod deny_some;
mod error_propagation;
mod error_response;
mod grant_all;
mod headers;
mod injection;
mod multiple;
mod query;
mod requires_scopes;
mod response;

pub use headers::*;

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
