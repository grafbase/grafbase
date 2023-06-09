use crate::common::{expect, expect_ts};
use grafbase_client_generator::expression::Value;

#[test]
fn string_value() {
    let value = Value::from("foo");

    let expected = expect![[r#"
            'foo'
        "#]];

    expect_ts(&value, &expected);
}

#[test]
fn float_value() {
    let value = Value::from(1.23f64);

    let expected = expect![[r#"
            1.23
        "#]];

    expect_ts(&value, &expected);
}

#[test]
fn rounded_float_value() {
    let value = Value::from(3.0f64);

    let expected = expect![[r#"
            3
        "#]];

    expect_ts(&value, &expected);
}

#[test]
fn nan_float_value() {
    let value = Value::from(f64::NAN);

    let expected = expect![[r#"
            NaN
        "#]];

    expect_ts(&value, &expected);
}

#[test]
fn infinite_float_value() {
    let value = Value::from(f64::INFINITY);

    let expected = expect![[r#"
            Infinity
        "#]];

    expect_ts(&value, &expected);
}

#[test]
fn neg_infinite_float_value() {
    let value = Value::from(f64::NEG_INFINITY);

    let expected = expect![[r#"
            ;-Infinity
        "#]];

    expect_ts(&value, &expected);
}
