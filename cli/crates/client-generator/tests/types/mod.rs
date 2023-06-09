use crate::common::{expect_raw_ts, expect_ts};
use expect_test::expect;
use grafbase_client_generator::{
    r#type::{MappedType, Property, StaticType, Type, TypeCondition, TypeGenerator},
    statement::Export,
};

#[test]
fn property_type_map() {
    let source = Property::new("key", StaticType::ident("string"));
    let definition = StaticType::ident("boolean").or(StaticType::ident("Horse"));
    let map = MappedType::new(source, definition);

    let expected = expect!["{ [key: string]: boolean | Horse }"];

    expect_raw_ts(&map, &expected);
}

#[test]
fn generator_type_map() {
    let mut ident = StaticType::ident("TruthyKeys");
    ident.push_param(StaticType::ident("S"));

    let source = TypeGenerator::new("P", ident);
    let definition = StaticType::ident("boolean").or(StaticType::ident("Horse"));
    let map = MappedType::new(source, definition);

    let expected = expect!["{ [P in TruthyKeys<S>]: boolean | Horse }"];

    expect_raw_ts(&map, &expected);
}

#[test]
fn keyof_generator_type_map() {
    let ident = StaticType::ident("Type").keyof();
    let source = TypeGenerator::new("Property", ident);
    let definition = StaticType::ident("boolean");
    let map = MappedType::new(source, definition);

    let expected = expect!["{ [Property in keyof Type]: boolean }"];

    expect_raw_ts(&map, &expected);
}

#[test]
fn type_map_in_condition() {
    let ident = StaticType::ident("Type").keyof();
    let source = TypeGenerator::new("Property", ident);
    let definition = StaticType::ident("boolean");
    let map = MappedType::new(source, definition);

    let mut record = StaticType::ident("Record");

    record.push_param(StaticType::ident("string"));
    record.push_param(StaticType::ident("string"));

    let u = StaticType::ident("U")
        .extends(record)
        .condition(TypeCondition::new(map, StaticType::ident("number")));

    let expected = expect!["U extends Record<string, string> ? { [Property in keyof Type]: boolean } : number"];

    expect_raw_ts(&u, &expected);
}

#[test]
fn basic_type_generator() {
    let mut ident = StaticType::ident("TruthyKeys");
    ident.push_param(StaticType::ident("S"));

    let gen = TypeGenerator::new("P", ident);

    let expected = expect!["P in TruthyKeys<S>"];

    expect_raw_ts(&gen, &expected);
}

#[test]
fn simple_type_ident() {
    let ident = StaticType::ident("BlogNode");
    let expected = expect![[r#"
            BlogNode
        "#]];

    expect_ts(&ident, &expected);
}

#[test]
fn type_ident_with_or() {
    let ident = StaticType::ident("string").or(StaticType::string("foo"));

    let expected = expect![[r#"
            string | 'foo'
        "#]];

    expect_ts(&ident, &expected);
}

#[test]
fn type_ident_with_params() {
    let mut ident = StaticType::ident("BlogNode");
    ident.push_param(StaticType::ident("T"));
    ident.push_param(StaticType::ident("U"));

    let expected = expect!["BlogNode<T, U>"];

    expect_raw_ts(&ident, &expected);
}

#[test]
fn type_ident_with_extends() {
    let mut record = StaticType::ident("Record");

    let key = StaticType::ident("string");

    let val = StaticType::ident("null")
        .or(StaticType::ident("boolean"))
        .or(StaticType::ident("object"));

    record.push_param(key);
    record.push_param(val);

    let u = StaticType::ident("U").extends(record);
    let expected = expect!["U extends Record<string, null | boolean | object>"];

    expect_raw_ts(&u, &expected);
}

#[test]
fn extends_keyof() {
    let blog_node = StaticType::ident("BlogNode").keyof();
    let u = StaticType::ident("P").extends(blog_node);

    let expected = expect!["P extends keyof BlogNode"];

    expect_raw_ts(&u, &expected);
}

#[test]
fn type_ident_with_extends_condition() {
    let mut record = StaticType::ident("Record");

    record.push_param(StaticType::ident("string"));

    let u = StaticType::ident("U").extends(record).condition(TypeCondition::new(
        StaticType::ident("string"),
        StaticType::ident("number"),
    ));

    let expected = expect!["U extends Record<string> ? string : number"];

    expect_raw_ts(&u, &expected);
}

#[test]
fn simple_type_definition() {
    let r#type = Type::new(
        StaticType::ident("OrderByDirection"),
        StaticType::string("ASC").or(StaticType::string("DESC")),
    );

    let expected = expect![[r#"
            type OrderByDirection = 'ASC' | 'DESC'
        "#]];

    expect_ts(&r#type, &expected);
}

#[test]
fn export_type_definition() {
    let r#type = Type::new(
        StaticType::ident("OrderByDirection"),
        StaticType::string("ASC").or(StaticType::string("DESC")),
    );

    let r#type = Export::new(r#type);

    let expected = expect![[r#"export type OrderByDirection = 'ASC' | 'DESC'"#]];
    expected.assert_eq(&r#type.to_string());
}
