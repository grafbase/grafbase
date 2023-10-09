use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn two_pk_ids() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreateMany(input: [{ id: 1 }, { id: 2 }]) {
                id
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(2, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreateMany": [
              {
                "id": 1
              },
              {
                "id": 2
              }
            ]
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn wrong_keys() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(5) NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreateMany(input: [{ id: 1 }, { id: 2, name: "Musti" }]) {
                id
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(0, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": null,
          "errors": [
            {
              "message": "All insert items must have the same columns.",
              "locations": [
                {
                  "line": 2,
                  "column": 3
                }
              ],
              "path": [
                "userCreateMany"
              ]
            }
          ]
        }"#]];

    expected.assert_eq(&response);
}
