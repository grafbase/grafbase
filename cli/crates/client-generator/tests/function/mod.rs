use grafbase_client_generator::{
    r#type::StaticType,
    statement::{Export, Return},
    Block, Function, Identifier,
};

use crate::common::{expect, expect_ts};

#[test]
fn basic_function() {
    let mut block = Block::new();
    block.push(Return::new(Identifier::new("foo")));

    let function = Function::new("bar", block)
        .push_param("foo", StaticType::ident("string"))
        .returns(StaticType::ident("string"));

    let expected = expect![[r#"
        function bar(foo: string): string {
          return foo
        }
    "#]];

    expect_ts(&function, &expected);
}

#[test]
fn export_function() {
    let mut block = Block::new();
    block.push(Return::new(Identifier::new("foo")));

    let function = Function::new("bar", block)
        .push_param("foo", StaticType::ident("string"))
        .returns(StaticType::ident("string"));

    let expected = expect![[r#"
        export function bar(foo: string): string {
          return foo
        }
    "#]];

    expect_ts(&Export::new(function), &expected);
}
