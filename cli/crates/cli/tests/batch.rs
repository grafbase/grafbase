#![allow(unused_crate_dependencies)]

mod utils;

use backend::project::ConfigType;
use serde_json::{json, Value};
use utils::client::Client;
use utils::consts::{BATCH_CCOLLECT, BATCH_CREATE, BATCH_SCHEMA, BATCH_UPDATE};
use utils::environment::Environment;

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
struct Collection {
    edges: Vec<Edge>,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
struct Edge {
    node: Post,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CreateUpdatePayload {
    post_collection: Vec<Post>,
}

impl CreateUpdatePayload {
    fn content_equal(&self, other: &CreateUpdatePayload) -> bool {
        self.post_collection.len() == other.post_collection.len() && {
            let mut a = self.post_collection.clone();
            let mut b = other.post_collection.clone();
            a.sort_by_key(|post| post.slug.clone());
            b.sort_by_key(|post| post.slug.clone());
            a.iter().zip(b.iter()).all(|(a, b)| a.content_equal(b))
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
struct Post {
    id: String,
    slug: String,
    author: Option<Author>,
}

impl Post {
    fn content_equal(&self, other: &Post) -> bool {
        self.slug == other.slug
            && match (&self.author, &other.author) {
                (Some(a), Some(b)) => a.name == b.name,
                (None, None) => true,
                _ => false,
            }
    }
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
struct Author {
    id: String,
    name: String,
}

fn all_posts(client: &Client) -> Collection {
    dot_get!(
        client
            .gql::<Value>(BATCH_CCOLLECT)
            .variables(json!({
                "first": 100
            }))
            .send(),
        "data.postCollection"
    )
}

macro_rules! assert_content_equal {
    ($result: expr, $expected: expr) => {
        let result = $result;
        let expected = $expected;
        assert!(
            result.content_equal(expected),
            "{:?}\n== not content_equal ==\n{:?}",
            result,
            expected
        );
    };
}

#[test]
fn batch_create() {
    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(BATCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let create_response: CreateUpdatePayload = dot_get!(
        client
            .gql::<Value>(BATCH_CREATE)
            .variables(json!({
                "input": [
                    { "input": { "slug": "Best Post Ever", "author": { "create": { "name": "Jamie" } } } },
                    { "input": { "slug": "The Bible" } }
                ]
            }))
            .send(),
        "data.postCreateMany"
    );
    assert_content_equal!(
        &create_response,
        &CreateUpdatePayload {
            post_collection: vec![
                Post {
                    id: String::new(),
                    slug: "Best Post Ever".to_string(),
                    author: Some(Author {
                        id: String::new(),
                        name: "Jamie".to_string()
                    })
                },
                Post {
                    id: String::new(),
                    slug: "The Bible".to_string(),
                    author: None
                }
            ]
        }
    );

    let posts = all_posts(&client);
    assert_eq!(
        posts,
        Collection {
            edges: create_response
                .post_collection
                .into_iter()
                .map(|post| Edge { node: post })
                .collect(),
        }
    );

    let response = client
        .gql::<Value>(BATCH_CREATE)
        .variables(json!({
            "input": [
                { "input": { "slug": "The Bible" } },
                { "input": { "slug": "The new stuff!" } }
            ]
        }))
        .send();

    // slug is unique
    assert!(!dot_get!(response, "errors", Vec<Value>).is_empty(), "{response:?}");
    assert!(dot_get_opt!(response, "data", Value).is_none());
    // Nothing was added
    assert_eq!(all_posts(&client), posts);

    let response = client
        .gql::<Value>(BATCH_CREATE)
        .variables(json!({
            "input": [
                { "input": { "slug": "The Latest Post", "author": { "create": { "name": "Jamie" } } } },
                { "input": { "slug": "Hot news!" } }
            ]
        }))
        .send();

    // author name is unique
    assert!(!dot_get!(response, "errors", Vec<Value>).is_empty(), "{response:?}");
    assert!(dot_get_opt!(response, "data", Value).is_none());
    // Nothing was added
    assert_eq!(all_posts(&client), posts);
}

#[test]
fn batch_update() {
    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(BATCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    print!(
        "{:#?}",
        client
        .gql::<Value>(BATCH_CREATE)
        .variables(json!({
            "input": [
                { "input": { "slug": "Best Post Ever", "author": { "create": { "name": "Jamie" } } } },
                { "input": { "slug": "The Bible" } },
                { "input": { "slug": "Hamlet", "author": { "create": { "name": "Some random englishman, not Jamie" } } } }
            ]
        }))
        .send()
    );
    let best_post_ever_id = all_posts(&client).edges.get(0).unwrap().node.id.clone();

    println!("{:#?}", all_posts(&client));
    let response: CreateUpdatePayload = dot_get!(
        client
            .gql::<Value>(BATCH_UPDATE)
            .variables(json!({
                "input": [
                    { "by": { "slug": "The Bible" }, "input": { "slug": "The Ancient Testament" } },
                    { "by": { "id": best_post_ever_id }, "input": { "slug": "The new stuff!" } }
                ]
            }))
            .send(),
        "data.postUpdateMany"
    );
    println!("{:#?}", all_posts(&client));
    assert_content_equal!(
        &response,
        &CreateUpdatePayload {
            post_collection: vec![
                Post {
                    id: String::new(),
                    slug: "The new stuff!".to_string(),
                    author: Some(Author {
                        id: String::new(),
                        name: "Jamie".to_string()
                    })
                },
                Post {
                    id: String::new(),
                    slug: "The Ancient Testament".to_string(),
                    author: None
                }
            ]
        }
    );
    let posts = all_posts(&client);

    // Slug is unique
    let response = client
        .gql::<Value>(BATCH_UPDATE)
        .variables(json!({
            "input": [
                // "Hamlet" exists already
                { "by": { "slug": "The Ancient Testament" }, "input": { "slug": "Hamlet" } },
                { "by": { "id": best_post_ever_id }, "input": { "slug": "Different" } }
            ]
        }))
        .send();
    assert!(!dot_get!(response, "errors", Vec<Value>).is_empty(), "{response:?}");
    assert!(dot_get_opt!(response, "data", Value).is_none());
    // Nothing was updated
    assert_eq!(all_posts(&client), posts);

    // Cannot update the same item multiple times.
    let response = client
        .gql::<Value>(BATCH_UPDATE)
        .variables(json!({
            "input": [
                { "by": { "slug": "The new stuff!" }, "input": { "slug": "Something" } },
                { "by": { "id": best_post_ever_id }, "input": { "slug": "Different" } }
            ]
        }))
        .send();
    assert!(!dot_get!(response, "errors", Vec<Value>).is_empty(), "{response:?}");
    assert!(dot_get_opt!(response, "data", Value).is_none());
    // Nothing was updated
    assert_eq!(all_posts(&client), posts);

    // Author name is unique
    let response = client
        .gql::<Value>(BATCH_UPDATE)
        .variables(json!({
            "input": [
                // "Hamlet" exists already
                { "by": { "slug": "The Ancient Testament" }, "input": { "author": { "create": { "name": "Jamie" } } } },
                { "by": { "id": best_post_ever_id }, "input": { "slug": "Different" } }
            ]
        }))
        .send();
    assert!(!dot_get!(response, "errors", Vec<Value>).is_empty(), "{response:?}");
    assert!(dot_get_opt!(response, "data", Value).is_none());
    // Nothing was updated
    assert_eq!(all_posts(&client), posts);
}
