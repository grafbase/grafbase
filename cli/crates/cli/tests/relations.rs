#![allow(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
mod utils;

use backend::project::GraphType;
use serde_json::{json, Value};
use utils::consts::{
    REALTIONS_LINK_SECONDARY_AUTHOR_TO_BLOG, REALTIONS_RENAME_AUTHOR, RELATIONS_LINK_BLOG_TO_AUTHOR,
    RELATIONS_MUTATION, RELATIONS_QUERY, RELATIONS_SCHEMA, RELATIONS_UNLINK_AUTHORS_FROM_BLOG,
    RELATIONS_UNLINK_BLOG_FROM_AUTHOR,
};
use utils::environment::Environment;

#[test]
fn relations() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(RELATIONS_SCHEMA);

    env.grafbase_dev_watch();

    let client = env.create_client().with_api_key();

    // wait for node to be ready
    client.poll_endpoint(30, 300);

    client.gql::<Value>(RELATIONS_MUTATION).send();

    let response = client.gql::<Value>(RELATIONS_QUERY).send();

    let blog: Value = dot_get!(response, "data.blogCollection.edges.0.node");
    let blog_id: String = dot_get!(blog, "id");
    let first_author_id: String = dot_get!(blog, "authors.edges.0.node.id");
    let second_author_id: String = dot_get!(blog, "authors.edges.1.node.id");
    let first_author_name: String = dot_get!(blog, "authors.edges.0.node.name");
    let first_authors_blogs: Vec<Value> = dot_get!(
        response,
        "data.blogCollection.edges.0.node.authors.edges.0.node.blogs.edges"
    );

    assert!(blog_id.starts_with("blog_"));
    assert_eq!(first_author_name, "1");
    assert!(first_authors_blogs.is_empty());

    client
        .gql::<Value>(RELATIONS_LINK_BLOG_TO_AUTHOR)
        .variables(json!({ "id": first_author_id, "blogId": blog_id}))
        .send();

    let response = client.gql::<Value>(RELATIONS_QUERY).send();

    let current_first_author_id: String =
        dot_get!(response, "data.blogCollection.edges.0.node.authors.edges.0.node.id");
    let first_authors_first_blog_id: Value = dot_get!(
        response,
        "data.blogCollection.edges.0.node.authors.edges.0.node.blogs.edges.0.node.id"
    );

    assert_eq!(current_first_author_id, first_author_id);
    assert_eq!(blog_id, first_authors_first_blog_id);
    assert_eq!(blog_id, first_authors_first_blog_id);

    client
        .gql::<Value>(RELATIONS_UNLINK_BLOG_FROM_AUTHOR)
        .variables(json!({ "id": first_author_id, "blogId": blog_id}))
        .send();

    let response = client.gql::<Value>(RELATIONS_QUERY).send();

    let current_first_author_id: String =
        dot_get!(response, "data.blogCollection.edges.0.node.authors.edges.0.node.id");
    let first_authors_blogs: Vec<Value> = dot_get!(
        response,
        "data.blogCollection.edges.0.node.authors.edges.0.node.blogs.edges"
    );

    assert_eq!(current_first_author_id, first_author_id);
    assert!(first_authors_blogs.is_empty());

    client
        .gql::<Value>(RELATIONS_LINK_BLOG_TO_AUTHOR)
        .variables(json!({ "id": first_author_id, "blogId": blog_id}))
        .send();

    client
        .gql::<Value>(REALTIONS_LINK_SECONDARY_AUTHOR_TO_BLOG)
        .variables(json!({ "id": blog_id, "authorId": first_author_id }))
        .send();

    let response = client
        .gql::<Value>(REALTIONS_RENAME_AUTHOR)
        .variables(json!({ "id": second_author_id, "name": "renamed" }))
        .send();

    let current_author_name: String = dot_get!(response, "data.authorUpdate.author.name");

    assert_eq!(current_author_name, "renamed");

    let response = client
        .gql::<Value>(RELATIONS_UNLINK_AUTHORS_FROM_BLOG)
        .variables(json!({
                "id": blog_id,
                "author1": first_author_id,
                "author2": second_author_id
        }))
        .send();

    let errors: Option<Value> = dot_get_opt!(response, "errors");

    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[test]
fn test_relation_unlinking() {
    const SCHEMA: &str = r"
        type Environment @model {
            groups: [Group]
        }
        type Group @model {
            environment: Environment
        }
    ";

    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(SCHEMA);

    env.grafbase_dev_watch();

    let client = env.create_client().with_api_key();

    // wait for node to be ready
    client.poll_endpoint(30, 300);

    let value = client
        .gql::<Value>(
            r"
            mutation CreateEnv {
                environmentCreate(input: {groups: {create: {}}}) {
                    environment {
                        id
                        groups(first: 10) {
                            edges {
                                node {
                                    id
                                }
                            }
                        }
                    }
                }
            }
        ",
        )
        .send();

    let env_id = dot_get!(value, "data.environmentCreate.environment.id", String);
    let group_id = dot_get!(
        value,
        "data.environmentCreate.environment.groups.edges.0.node.id",
        String
    );

    let result = client
        .gql::<Value>(
            r"
            mutation UnlinkGroupFromEnv($groupId: ID!, $envId: ID) {
                groupUpdate(
                  by: {
                    id: $groupId
                  }
                  input: { environment: { unlink: $envId} }
                ) {
                  group {
                    id
                    environment {
                        id
                    }
                  }
                }
            }
        ",
        )
        .variables(serde_json::json!({ "groupId": group_id, "envId": env_id }))
        .send();

    assert_eq!(
        result,
        serde_json::json!({
            "data": {"groupUpdate": {"group": {"id": group_id, "environment": null}}}
        })
    );

    let result = client
        .gql::<Value>(
            r"
            query GetGroup($groupId: ID!) {
                group(by: {id: $groupId}) {
                    id
                    environment {
                        id
                    }
                }
            }
        ",
        )
        .variables(serde_json::json!({ "groupId": group_id }))
        .send();

    assert_eq!(
        result,
        serde_json::json!({
            "data": {"group": {"id": group_id, "environment": null}}
        })
    );
}

#[test]
fn test_relation_unlink_and_create() {
    const SCHEMA: &str = r"
        type Environment @model {
            groups: [Group]
        }
        type Group @model {
            environment: Environment
        }
    ";

    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(SCHEMA);

    env.grafbase_dev_watch();

    let client = env.create_client().with_api_key();

    // wait for node to be ready
    client.poll_endpoint(30, 300);

    let value = client
        .gql::<Value>(
            r"
            mutation CreateEnv {
                environmentCreate(input: {groups: {create: {}}}) {
                    environment {
                        id
                        groups(first: 10) {
                            edges {
                                node {
                                    id
                                }
                            }
                        }
                    }
                }
            }
        ",
        )
        .send();

    let env_id = dot_get!(value, "data.environmentCreate.environment.id", String);
    let group_id = dot_get!(
        value,
        "data.environmentCreate.environment.groups.edges.0.node.id",
        String
    );

    let result = client
        .gql::<Value>(
            r"
            mutation EnvUpdate($groupId: ID!, $envId: ID) {
                environmentUpdate(
                  by: {
                    id: $envId
                  }
                  input: { groups: [{ unlink: $groupId }, {create: {}}] }
                ) {
                    environment {
                        id
                        groups(first: 10) {
                            edges {
                                node {
                                    id
                                }
                            }
                        }
                    }
                }
            }
        ",
        )
        .variables(serde_json::json!({ "groupId": group_id, "envId": env_id }))
        .send();

    let groups = dot_get!(result, "data.environmentUpdate.environment.groups.edges", Vec<Value>);
    assert_eq!(groups.len(), 1);

    let new_group_id = dot_get!(
        result,
        "data.environmentUpdate.environment.groups.edges.0.node.id",
        String
    );

    assert_ne!(new_group_id, group_id);

    let result = client
        .gql::<Value>(
            r"
            query GetGroup($groupId: ID!) {
                group(by: {id: $groupId}) {
                    id
                    environment {
                        id
                    }
                }
            }
        ",
        )
        .variables(serde_json::json!({ "groupId": new_group_id }))
        .send();

    assert_eq!(
        result,
        serde_json::json!({
            "data": {"group": {"id": new_group_id, "environment": {"id": env_id}}}
        })
    );
}

#[test]
fn update_bug_gb4646() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(
        r"
        type Player @model {
          name: String! @unique
          notes: [Note]
        }

        type Note @model {
          note: String!
          player: Player!
        }
    ",
    );
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let resp = client
        .gql::<Value>(
            r#"
            mutation PlayerCreate {
              playerCreate(input: {name: "freddie"}) {
                player {
                  id
                }
              }
            }
            "#,
        )
        .send();
    let player_id: String = dot_get!(resp, "data.playerCreate.player.id");

    let resp = client
        .gql::<Value>(
            r#"
            mutation NotesCreate($id: ID!) {
              noteCreate(input: {note: "first", player: {link: $id}}) {
                note {
                  id
                  note
                }
              }
            }
            "#,
        )
        .variables(json!({
            "id": player_id
        }))
        .send();
    let note_id: String = dot_get!(resp, "data.noteCreate.note.id");

    let update = |note: &str| {
        client
            .gql::<Value>(
                r"
                mutation NotesUpdate($id: ID!, $note: String!) {
                  noteUpdate(by: {id: $id}, input: {note: $note}) {
                    note {
                      id
                    }
                  }
                }
                ",
            )
            .variables(json!({
                "id": note_id,
                "note": note
            }))
            .send();
    };
    let get_relation_note = || -> String {
        dot_get!(
            client
                .gql::<Value>(
                    r"
                query ListPlayers {
                  playerCollection(first: 100) {
                    edges {
                      node {
                        id
                        name
                        notes(first: 100) {
                          edges {
                            node {
                              id
                              note
                              player {
                                id
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
                ",
                )
                .send(),
            "data.playerCollection.edges.0.node.notes.edges.0.node.note"
        )
    };
    update("second");
    assert_eq!(get_relation_note(), "second");

    update("third");
    assert_eq!(get_relation_note(), "third");
}
