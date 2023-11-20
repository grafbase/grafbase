use expect_test::expect;
use indoc::indoc;
use integration_tests::{with_mongodb, with_namespaced_mongodb};

#[test]
fn not_namespaced() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreateMany(input: [
                { id: "5ca4bbc7a2dd94ee5816238d" }
              ]) {
                insertedIds
              }
            }
        "#};

        api.execute(mutation).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreateMany": {
              "insertedIds": [
                "5ca4bbc7a2dd94ee5816238d"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let schema = indoc! {r#"
        type User @model(connector: "mongo", collection: "users") {
          age: Int
        }
    "#};

    let response = with_namespaced_mongodb("mongo", schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              mongo {
                userCreateMany(input: [
                  { id: "5ca4bbc7a2dd94ee5816238e" }
                ]) {
                  insertedIds
                }
              }
            }
        "#};

        api.execute(mutation).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "mongo": {
              "userCreateMany": {
                "insertedIds": [
                  "5ca4bbc7a2dd94ee5816238e"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn with_renamed_data() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int! @map(name: "renamed" )
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r"
            mutation {
              userCreateMany(input: [
                { age: 30 },
                { age: 31 }
              ]) {
                insertedIds
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { age } }}
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "age": 30
                  }
                },
                {
                  "node": {
                    "age": 31
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
