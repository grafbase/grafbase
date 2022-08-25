mod utils;

use serde_json::{json, Value};
use utils::consts::{
    REALTIONS_LINK_SECONDARY_AUTHOR_TO_BLOG, REALTIONS_RENAME_AUTHOR, RELATIONS_LINK_BLOG_TO_AUTHOR,
    RELATIONS_MUTATION, RELATIONS_QUERY, RELATIONS_SCHEMA, RELATIONS_UNLINK_AUTHORS_FROM_BLOG,
    RELATIONS_UNLINK_BLOG_FROM_AUTHOR,
};
use utils::environment::Environment;

#[test]
fn relations() {
    let mut env = Environment::init(4002);

    env.grafbase_init();

    env.write_schema(RELATIONS_SCHEMA);

    env.grafbase_dev_watch();

    let client = env.create_client();

    // wait for node to be ready
    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": RELATIONS_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": RELATIONS_QUERY }).to_string());

    let blog: Value = dot_get!(response, "data.blogCollection.edges.0.node");
    let blog_id: String = dot_get!(blog, "id");
    let first_author_id: String = dot_get!(blog, "authors.0.id");
    let second_author_id: String = dot_get!(blog, "authors.1.id");
    let first_author_name: String = dot_get!(blog, "authors.0.name");
    let first_authors_blogs: Vec<Value> = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.blogs");

    assert!(blog_id.starts_with("blog_"));
    // latest first
    assert_eq!(first_author_name, "5");
    assert!(first_authors_blogs.is_empty());

    client.gql::<Value>(
        json!({
            "query": RELATIONS_LINK_BLOG_TO_AUTHOR,
            "variables": { "id": first_author_id, "blogId": blog_id}
        })
        .to_string(),
    );

    let response = client.gql::<Value>(json!({ "query": RELATIONS_QUERY }).to_string());

    let current_first_author_id: String = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.id");
    let first_authors_first_blog_id: Value =
        dot_get!(response, "data.blogCollection.edges.0.node.authors.0.blogs.0.id");

    assert_eq!(current_first_author_id, first_author_id);
    assert_eq!(blog_id, first_authors_first_blog_id);
    assert_eq!(blog_id, first_authors_first_blog_id);

    client.gql::<Value>(
        json!({
            "query": RELATIONS_UNLINK_BLOG_FROM_AUTHOR,
            "variables": { "id": first_author_id, "blogId": blog_id}
        })
        .to_string(),
    );

    let response = client.gql::<Value>(json!({ "query": RELATIONS_QUERY }).to_string());

    let current_first_author_id: String = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.id");
    let first_authors_blogs: Vec<Value> = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.blogs");

    assert_eq!(current_first_author_id, first_author_id);
    assert!(first_authors_blogs.is_empty());

    client.gql::<Value>(
        json!({
            "query": RELATIONS_LINK_BLOG_TO_AUTHOR,
            "variables": { "id": first_author_id, "blogId": blog_id}
        })
        .to_string(),
    );

    client.gql::<Value>(
        json!({
            "query": REALTIONS_LINK_SECONDARY_AUTHOR_TO_BLOG,
            "variables": { "id": blog_id, "authorId": first_author_id }
        })
        .to_string(),
    );

    let response = client.gql::<Value>(
        json!({
            "query": REALTIONS_RENAME_AUTHOR,
            "variables": { "id": second_author_id, "name": "renamed"  }
        })
        .to_string(),
    );

    let current_author_name: String = dot_get!(response, "data.authorUpdate.author.name");

    assert_eq!(current_author_name, "renamed");

    let response = client.gql::<Value>(
        json!({
            "query": RELATIONS_UNLINK_AUTHORS_FROM_BLOG,
            "variables": {
                "id": blog_id,
                "author1": first_author_id,
                "author2": second_author_id
            }
        })
        .to_string(),
    );

    let errors: Option<Value> = dot_get_opt!(response, "errors");

    assert!(errors.is_none(), "errors: {:#?}", errors);
}
