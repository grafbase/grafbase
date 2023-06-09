use crate::common::{expect, expect_ts};
use grafbase_client_generator::{
    class::{ClassProperty, Constructor, Method},
    r#type::{StaticType, TypeIdentifier},
    statement::{Assignment, Return},
    Block, Class, Identifier,
};

#[test]
fn basic_class() {
    let mut ident = TypeIdentifier::ident("Query").extends(StaticType::ident("Operation"));
    ident.push_param(StaticType::ident("T"));
    ident.push_param(StaticType::ident("U").extends(StaticType::ident("object")));

    let mut class = Class::new(ident);
    class.push_property(ClassProperty::new("collection", StaticType::ident("string")));

    let mut fetch_input = StaticType::ident("FetchInput");
    fetch_input.push_param(StaticType::ident("T"));
    fetch_input.push_param(StaticType::ident("U"));

    class.push_property(ClassProperty::new("input", fetch_input.clone()));

    let mut block = Block::new();
    block.push(Assignment::new("this.collection", Identifier::new("collection")));
    block.push(Assignment::new("this.input", Identifier::new("input")));

    let mut constructor = Constructor::new(block);
    constructor.push_param("collection", StaticType::ident("string"));
    constructor.push_param("input", fetch_input.clone());

    class.set_constructor(constructor);

    let mut block = Block::new();
    block.push(Return::new(Identifier::new("this.input")));

    let method = Method::new("getInput", block).returns(fetch_input);
    class.push_method(method);

    let expected = expect![[r#"
        class Query<T, U extends object> extends Operation {
          collection: string
          input: FetchInput<T, U>

          constructor(collection: string, input: FetchInput<T, U>) {
            this.collection = collection
            this.input = input
          }

          getInput(): FetchInput<T, U> {
            return this.input
          }
        }
    "#]];

    expect_ts(&class, &expected);
}
