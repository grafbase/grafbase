use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn char() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val CHAR(5) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn name() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val NAME NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn text() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TEXT NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn xml() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val XML NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '<html></html>')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "<html></html>"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn cidr() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val CIDR NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '0.0.0.0/0')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "0.0.0.0/0"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr8() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val MACADDR8 NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '08:00:2b:01:02:03:04:05')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "08:00:2b:01:02:03:04:05"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val MACADDR NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '08:00:2b:01:02:03')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "08:00:2b:01:02:03"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bpchar() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BPCHAR(5) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varchar() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val VARCHAR(5) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bit() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BIT(3) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, B'010')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "010"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varbit() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val VARBIT(3) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, B'010')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "010"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn xml_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val XML[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{<html></html>, <head></head>}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "<html></html>",
                "<head></head>"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn cidr_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val CIDR[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{0.0.0.0/0, 192.168.0.0/32}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "0.0.0.0/0",
                "192.168.0.0/32"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr8_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val MACADDR8[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{08:00:2b:01:02:03:04:05, 08002b:0102030405}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "08:00:2b:01:02:03:04:05",
                "08:00:2b:01:02:03:04:05"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn macaddr_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val MACADDR8[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{08:00:2b:01:02:03:04:05, 08002b:0102030405}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "08:00:2b:01:02:03:04:05",
                "08:00:2b:01:02:03:04:05"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn char_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val char(6)[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{Musti, Naukio}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "Musti ",
                "Naukio"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn name_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val NAME[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{Musti, Naukio}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "Musti",
                "Naukio"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn text_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TEXT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{Musti, Naukio}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "Musti",
                "Naukio"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bpchar_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BPCHAR(6)[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{Musti, Naukio}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "Musti ",
                "Naukio"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varchar_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val VARCHAR(6)[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{Musti, Naukio}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "Musti",
                "Naukio"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bit_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BIT(3)[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{010, 110}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "010",
                "110"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn varbit_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val VARBIT(3)[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{010, 110}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "010",
                "110"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int8() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INT8 NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 420)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "420"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn oid() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val OID NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 420)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "420"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INT2 NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 420)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": 420
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int4() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INT4 NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 420)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": 420
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn json() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val JSON NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{ "foo": 1 }')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": {
                "foo": 1
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
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val JSONB NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{ "foo": 1 }')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": {
                "foo": 1
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
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val JSON[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['{"foo":1}','{"bar": false}']::json[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                {
                  "foo": 1
                },
                {
                  "bar": false
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val JSONB[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['{"foo":1}','{"bar": false}']::jsonb[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                {
                  "foo": 1
                },
                {
                  "bar": false
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn money() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val MONEY NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '1.23')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "$1.23"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn numeric() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val NUMERIC NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '1.23')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "1.23"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn money_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val MONEY[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1.23, 3.14}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "$1.23",
                "$3.14"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn numeric_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val NUMERIC[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1.23, 3.14}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "1.23",
                "3.14"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INT2[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1, 2}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                1,
                2
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int4_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INT4[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1, 2}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                1,
                2
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float4_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val FLOAT4[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1.24, 3.0}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                1.24,
                3.0
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float8_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val FLOAT8[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1.24, 3.0}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                1.24,
                3.0
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn time() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIME NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '16:20:00')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "16:20:00"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timetz() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIMETZ NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '16:20:00Z')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "16:20:00+00"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int8_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INT8[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1, 2}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "1",
                "2"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn oid_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val OID[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{1, 2}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "1",
                "2"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float4() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val FLOAT4 NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 1.23)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": 1.23
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float8() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val FLOAT8 NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 1.23)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": 1.23
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn time_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIME[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{16:20:00, 04:20:00}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "16:20:00",
                "04:20:00"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timetz_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIMETZ[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{16:20:00Z, 04:20:00Z}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "16:20:00+00",
                "04:20:00+00"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bool() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BOOL NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, true)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": true
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bytea() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BYTEA NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'DEADBEEF'::bytea)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "REVBREJFRUY"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inet() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INET NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '192.168.0.1')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "192.168.0.1"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bool_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BOOL[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{true, false}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                true,
                false
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bytea_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val BYTEA[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['DEADBEEF'::bytea, 'BEEFDEAD'::bytea]::bytea[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "REVBREJFRUY",
                "QkVFRkRFQUQ"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inet_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val INET[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{192.168.0.1, 192.168.0.2}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "192.168.0.1",
                "192.168.0.2"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn date() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val DATE NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '1999-01-08')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "1999-01-08"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamp() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIMESTAMP NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '2004-10-19T10:23:54')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "2004-10-19T10:23:54"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamp_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIMESTAMP[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['2004-10-19T10:23:54', '2004-10-19T10:23:55']::timestamp[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "2004-10-19T10:23:54",
                "2004-10-19T10:23:55"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn date_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val DATE[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['1999-01-08', '2011-09-11']::date[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "1999-01-08",
                "2011-09-11"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamptz() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIMESTAMPTZ NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '2004-10-19T10:23:54.000Z')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "2004-10-19T10:23:54.000Z"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamptz_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val TIMESTAMPTZ[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['2004-10-19T10:23:54Z', '2004-10-19T10:23:55Z']::timestamptz[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "2004-10-19T10:23:54.000Z",
                "2004-10-19T10:23:55.000Z"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn uuid() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val UUID NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'd89bd15d-ac64-4c71-895c-adba9c35a132')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "d89bd15d-ac64-4c71-895c-adba9c35a132"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn uuid_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "A" (
                id INT PRIMARY KEY,
                val UUID[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, array['d89bd15d-ac64-4c71-895c-adba9c35a132']::uuid[])
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "d89bd15d-ac64-4c71-895c-adba9c35a132"
              ]
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

        let table = indoc! {r#"
            CREATE TABLE "A" (
              id INT PRIMARY KEY,
              val street_light NOT NULL
            );
        "#};

        api.execute_sql(table).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, 'yellow')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": "YELLOW"
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

        let table = indoc! {r#"
            CREATE TABLE "A" (
              id INT PRIMARY KEY,
              val street_light[] NOT NULL
            );
        "#};

        api.execute_sql(table).await;

        let insert = indoc! {r#"
            INSERT INTO "A" (id, val) VALUES (1, '{yellow, red, green}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              a(by: { id: 1 }) { id val }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "a": {
              "id": 1,
              "val": [
                "YELLOW",
                "RED",
                "GREEN"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
