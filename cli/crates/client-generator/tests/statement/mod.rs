use grafbase_client_generator::{expression::Value, statement::Conditional, Block};

use crate::common::{expect, expect_ts};

#[test]
fn single_if() {
    let mut block = Block::new();
    block.push(Value::from(1));

    let conditional = Conditional::new(Value::from(true), block);

    let expected = expect![[r#"
        if (true) {
          1
        }
    "#]];

    expect_ts(&conditional, &expected);
}

#[test]
fn if_else() {
    let mut block = Block::new();
    block.push(Value::from(1));

    let mut conditional = Conditional::new(Value::from(true), block);

    let mut block = Block::new();
    block.push(Value::from(2));

    conditional.r#else(block);

    let expected = expect![[r#"
            if (true) {
              1
            } else {
              2
            }
        "#]];

    expect_ts(&conditional, &expected);
}

#[test]
fn if_else_if_else() {
    let mut block = Block::new();
    block.push(Value::from(1));

    let mut conditional = Conditional::new(Value::from(true), block);

    let mut block = Block::new();
    block.push(Value::from(2));

    conditional.else_if(Value::from(false), block);

    let mut block = Block::new();
    block.push(Value::from(3));

    conditional.r#else(block);

    let expected = expect![[r#"
            if (true) {
              1
            } else if (false) {
              2
            } else {
              3
            }
        "#]];

    expect_ts(&conditional, &expected);
}
