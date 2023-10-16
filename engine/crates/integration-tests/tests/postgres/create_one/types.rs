use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn char() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val CHAR(5) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "Musti" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "Musti"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn char_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val CHAR(6)[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["Musti", "Naukio"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "Musti ",
                  "Naukio"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn name() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val NAME NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "Musti" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "Musti"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn name_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val NAME[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["Musti", "Naukio"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "Musti",
                  "Naukio"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn text() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TEXT NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "Musti" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "Musti"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn text_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TEXT[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["Musti", "Naukio"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "Musti",
                  "Naukio"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn xml() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val XML NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "<html></html>" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "<html></html>"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn xml_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val XML[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["<html></html>", "<head></head>"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "<html></html>",
                  "<head></head>"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn cidr() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val CIDR NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "0.0.0.0/0" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "0.0.0.0/0"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn cidr_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val CIDR[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["0.0.0.0/0", "192.168.0.0/32"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "0.0.0.0/0",
                  "192.168.0.0/32"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr8() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val MACADDR8 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "08:00:2b:01:02:03:04:05" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "08:00:2b:01:02:03:04:05"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr8_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val MACADDR8[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["08:00:2b:01:02:03:04:05", "08002b:0102030405"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "08:00:2b:01:02:03:04:05",
                  "08:00:2b:01:02:03:04:05"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val MACADDR NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "08:00:2b:01:02:03" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "08:00:2b:01:02:03"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val MACADDR[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["08:00:2b:01:02:03", "08:00:2b:01:02:04"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "08:00:2b:01:02:03",
                  "08:00:2b:01:02:04"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bpchar() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BPCHAR(5) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "Musti" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "Musti"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bpchar_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BPCHAR(6)[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["Musti", "Naukio"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "Musti ",
                  "Naukio"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varchar() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val VARCHAR(5) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "Musti" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "Musti"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varchar_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val VARCHAR(6)[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["Musti", "Naukio"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "Musti",
                  "Naukio"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bit() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BIT(3) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "010" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "010"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bit_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BIT(3)[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["010", "101"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "010",
                  "101"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varbit() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val VARBIT(3) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "010" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "010"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varbit_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val VARBIT(3)[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["010", "101"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "010",
                  "101"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INT2 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: 420 }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": 420
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INT2[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [1, 2] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  1,
                  2
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int4() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INT4 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: 420 }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": 420
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int4_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INT4[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [1, 2] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  1,
                  2
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int8() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INT8 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "420" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "420"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int8_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INT8[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["1", "2"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "1",
                  "2"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn oid() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val OID NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "420" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "420"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn oid_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val OID[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["1", "2"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "1",
                  "2"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn json() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val JSON NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: { foo: 1 } }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": {
                  "foo": 1
                }
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn json_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val JSON[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [{ foo: 1 }, { bar: 2 }] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  {
                    "foo": 1
                  },
                  {
                    "bar": 2
                  }
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val JSONB NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: { foo: 1 } }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": {
                  "foo": 1
                }
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val JSONB[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [{ foo: 1 }, { bar: 2 }] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  {
                    "foo": 1
                  },
                  {
                    "bar": 2
                  }
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn money() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val MONEY NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "1.23" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "$1.23"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn money_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val MONEY[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["1.23", "3.14"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "$1.23",
                  "$3.14"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn numeric() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val NUMERIC NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "1.23" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "1.23"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn numeric_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val NUMERIC[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["1.23", "3.14"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "1.23",
                  "3.14"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float4() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val FLOAT4 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: 3.14 }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": 3.14
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float4_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val FLOAT4[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [3.14, 1.23] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  3.14,
                  1.23
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float8() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val FLOAT8 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: 3.14 }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": 3.14
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float8_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val FLOAT8[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [3.14, 1.23] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  3.14,
                  1.23
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn time() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIME NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "16:20:00" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "16:20:00"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn time_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIME[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["16:20:00", "04:20:00"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "16:20:00",
                  "04:20:00"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timetz() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIMETZ NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "16:20:00+00" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "16:20:00+00"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timetz_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIMETZ[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["16:20:00+00", "04:20:00Z"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "16:20:00+00",
                  "04:20:00+00"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bool() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BOOL NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: true }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": true
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bool_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BOOL[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [true, false] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  true,
                  false
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bytea() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BYTEA NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "XHg0NDQ1NDE0NDQyNDU0NTQ2" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "XHg0NDQ1NDE0NDQyNDU0NTQ2"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bytea_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val BYTEA[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["XHg0NDQ1NDE0NDQyNDU0NTQ2", "XHg0NDQ1NDE0NDQyNDU0NTQ3"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "XHg0NDQ1NDE0NDQyNDU0NTQ2",
                  "XHg0NDQ1NDE0NDQyNDU0NTQ3"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inet() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INET NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "192.168.0.1" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "192.168.0.1"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inet_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val INET[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["192.168.0.1", "10.0.0.1"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "192.168.0.1",
                  "10.0.0.1"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn date() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val DATE NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "1999-01-08" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "1999-01-08"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn date_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val DATE[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["1999-01-08", "1999-01-09"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "1999-01-08",
                  "1999-01-09"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamp() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIMESTAMP NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "2004-10-19T10:23:54" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "2004-10-19T10:23:54"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamp_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIMESTAMP[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["2004-10-19T10:23:54", "2004-10-19T10:23:55"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "2004-10-19T10:23:54",
                  "2004-10-19T10:23:55"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamptz() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIMESTAMPTZ NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "2004-10-19T10:23:54.000Z" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "2004-10-19T10:23:54.000Z"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamptz_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val TIMESTAMPTZ[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["2004-10-19T10:23:54.000Z", "2004-10-19T10:23:55.000Z"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "2004-10-19T10:23:54.000Z",
                  "2004-10-19T10:23:55.000Z"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn uuid() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val UUID NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: "d89bd15d-ac64-4c71-895c-adba9c35a132" }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "d89bd15d-ac64-4c71-895c-adba9c35a132"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn uuid_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val UUID[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: ["d89bd15d-ac64-4c71-895c-adba9c35a132", "d89bd15d-ac64-4c71-895c-adba9c35a133"] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "d89bd15d-ac64-4c71-895c-adba9c35a132",
                  "d89bd15d-ac64-4c71-895c-adba9c35a133"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn r#enum() {
    let response = query_postgres(|api| async move {
        let r#type = indoc! {r#"
            CREATE TYPE street_light AS ENUM ('red', 'yellow', 'green');
        "#};

        api.execute_sql(r#type).await;

        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val street_light NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: YELLOW }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": "YELLOW"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn enum_array() {
    let response = query_postgres(|api| async move {
        let r#type = indoc! {r#"
            CREATE TYPE street_light AS ENUM ('red', 'yellow', 'green');
        "#};

        api.execute_sql(r#type).await;

        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                val street_light[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { val: [YELLOW, GREEN] }) {
                returning { val }
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "returning": {
                "val": [
                  "YELLOW",
                  "GREEN"
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
