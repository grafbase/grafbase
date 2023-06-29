mod construct;
mod constructor;
mod method;
mod privacy;
mod property;

use std::fmt;

pub use construct::Construct;
pub use constructor::Constructor;
pub use method::Method;
pub use privacy::Privacy;
pub use property::ClassProperty;

use crate::r#type::TypeIdentifier;

pub struct Class<'a> {
    identifier: TypeIdentifier<'a>,
    properties: Vec<ClassProperty<'a>>,
    constructor: Option<Constructor<'a>>,
    methods: Vec<Method<'a>>,
}

#[allow(dead_code)]
impl<'a> Class<'a> {
    #[must_use]
    pub fn new(identifier: TypeIdentifier<'a>) -> Self {
        Self {
            identifier,
            properties: Vec::new(),
            constructor: None,
            methods: Vec::new(),
        }
    }

    pub fn set_constructor(&mut self, constructor: Constructor<'a>) {
        self.constructor = Some(constructor);
    }

    pub fn push_property(&mut self, property: ClassProperty<'a>) {
        self.properties.push(property);
    }

    pub fn push_method(&mut self, method: Method<'a>) {
        self.methods.push(method);
    }
}

impl<'a> fmt::Display for Class<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class {} {{", self.identifier)?;

        for property in &self.properties {
            writeln!(f, "{property}")?;
        }

        if let Some(ref constructor) = self.constructor {
            writeln!(f)?;
            constructor.fmt(f)?;
        }

        for method in &self.methods {
            writeln!(f)?;
            writeln!(f)?;
            method.fmt(f)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{expect, expect_ts};
    use crate::{
        class::{ClassProperty, Construct, Constructor, Method},
        expression::{Object, Value},
        r#type::{StaticType, TypeIdentifier},
        statement::{Assignment, Return},
        Block, Class, Identifier,
    };

    #[test]
    fn basic_class() {
        let mut ident = TypeIdentifier::ident("Query");
        ident.extends(StaticType::ident("Operation"));
        ident.push_param(StaticType::ident("T"));

        let mut u = StaticType::ident("U");
        u.extends(StaticType::ident("object"));
        ident.push_param(u);

        let mut class = Class::new(ident);
        class.push_property(ClassProperty::new("collection", StaticType::ident("string")));

        let mut fetch_input = TypeIdentifier::ident("FetchInput");
        fetch_input.push_param(StaticType::ident("T"));
        fetch_input.push_param(StaticType::ident("U"));

        class.push_property(ClassProperty::new("input", StaticType::new(fetch_input.clone())));

        let mut block = Block::new();
        block.push(Assignment::new("this.collection", Identifier::new("collection")));
        block.push(Assignment::new("this.input", Identifier::new("input")));

        let mut constructor = Constructor::new(block);
        constructor.push_param("collection", StaticType::ident("string"));
        constructor.push_param("input", StaticType::new(fetch_input.clone()));

        class.set_constructor(constructor);

        let mut block = Block::new();
        block.push(Return::new(Identifier::new("this.input")));

        let method = Method::new("getInput", block).returns(StaticType::new(fetch_input));
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

        dbg!(class.to_string());

        expect_ts(&class, &expected);
    }

    #[test]
    fn construct_new_instance() {
        let mut input = Object::new();
        input.entry("id", Value::from(1));

        let mut construct = Construct::new("Query");
        construct.push_param(Value::from("user"));
        construct.push_param(input);

        let expected = expect![[r#"
            new Query('user', { id: 1 })
        "#]];

        expect_ts(&construct, &expected);
    }
}
