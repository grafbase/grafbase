use crate::common::{expect, expect_ts};
use grafbase_client_generator::{
    expression::{Object, Value},
    r#type::{Property, StaticType},
    statement::{Assignment, Statement},
    Block, BlockItem, Interface,
};

#[test]
fn basic_block() {
    let mut block = Block::new();

    let mut interface = Interface::new("User");
    interface.push_property(Property::new("id", StaticType::ident("number")));
    interface.push_property(Property::new("name", StaticType::ident("string")));
    block.push(interface);

    block.push(BlockItem::newline());

    let mut object = Object::new();
    object.entry("id", Value::from(1));
    object.entry("name", Value::from("Naukio"));

    let assignment = Assignment::new("myObject", object)
        .r#const()
        .r#type(StaticType::ident("User"));

    block.push(Statement::from(assignment));

    let assignment = Assignment::new("foo", Value::from(1)).r#let();
    block.push(Statement::from(assignment));

    let assignment = Assignment::new("bar", Value::from(1)).var();
    block.push(Statement::from(assignment));

    let assignment = Assignment::new("bar", Value::from(2));
    block.push(Statement::from(assignment));

    let expected = expect![[r#"
        {
          interface User {
            id: number
            name: string
          }

          const myObject: User = { id: 1, name: 'Naukio' }
          let foo = 1
          var bar = 1
          bar = 2
        }
    "#]];

    expect_ts(&block, &expected);
}
