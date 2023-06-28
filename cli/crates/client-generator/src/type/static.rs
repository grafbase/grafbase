use std::{borrow::Cow, fmt};

use async_graphql_parser::types::BaseType;

use super::{TypeCondition, TypeIdentifier};

#[derive(Clone)]
pub struct StaticType<'a> {
    identifier: TypeIdentifier<'a>,
    or: Vec<StaticType<'a>>,
    condition: Option<Box<TypeCondition<'a>>>,
    keyof: bool,
}

impl<'a> StaticType<'a> {
    pub fn ident(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            identifier: TypeIdentifier::ident(name),
            or: Vec::new(),
            condition: None,
            keyof: false,
        }
    }

    pub fn string(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            identifier: TypeIdentifier::string(name),
            or: Vec::new(),
            condition: None,
            keyof: false,
        }
    }

    pub fn null() -> Self {
        Self::ident("null")
    }

    pub fn extends(&mut self, ident: StaticType<'a>) {
        self.identifier.extends(ident);
    }

    pub fn or(&mut self, ident: StaticType<'a>) {
        self.or.push(ident);
    }

    pub fn condition(&mut self, condition: TypeCondition<'a>) {
        self.condition = Some(Box::new(condition));
    }

    pub fn keyof(&mut self) {
        self.keyof = true;
    }

    pub fn from_graphql(base: &'a BaseType) -> Self {
        Self {
            identifier: TypeIdentifier::from_graphql(base),
            or: Vec::new(),
            condition: None,
            keyof: false,
        }
    }

    pub fn push_param(&mut self, param: StaticType<'a>) {
        self.identifier.push_param(param);
    }
}

impl<'a> fmt::Display for StaticType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.keyof {
            f.write_str("keyof ")?;
        }

        self.identifier.fmt(f)?;

        if !self.or.is_empty() {
            f.write_str(" | ")?;

            for (i, ident) in self.or.iter().enumerate() {
                ident.fmt(f)?;

                if i < self.or.len() - 1 {
                    f.write_str(" | ")?;
                }
            }
        }

        if let Some(ref condition) = self.condition {
            write!(f, " {condition}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{expect, expect_raw_ts, expect_ts};
    use crate::{
        r#type::{MappedType, Property, StaticType, Type, TypeCondition, TypeGenerator},
        statement::Export,
    };

    #[test]
    fn property_type_map() {
        let source = Property::new("key", StaticType::ident("string"));
        let mut definition = StaticType::ident("boolean");
        definition.or(StaticType::ident("Horse"));
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [key: string]: boolean | Horse }"];

        expect_raw_ts(&map, &expected);
    }

    #[test]
    fn generator_type_map() {
        let mut ident = StaticType::ident("TruthyKeys");
        ident.push_param(StaticType::ident("S"));

        let source = TypeGenerator::new("P", ident);
        let mut definition = StaticType::ident("boolean");
        definition.or(StaticType::ident("Horse"));
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [P in TruthyKeys<S>]: boolean | Horse }"];

        expect_raw_ts(&map, &expected);
    }

    #[test]
    fn keyof_generator_type_map() {
        let mut ident = StaticType::ident("Type");
        ident.keyof();

        let source = TypeGenerator::new("Property", ident);
        let definition = StaticType::ident("boolean");
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [Property in keyof Type]: boolean }"];

        expect_raw_ts(&map, &expected);
    }

    #[test]
    fn type_map_in_condition() {
        let mut ident = StaticType::ident("Type");
        ident.keyof();

        let source = TypeGenerator::new("Property", ident);
        let definition = StaticType::ident("boolean");
        let map = MappedType::new(source, definition);

        let mut record = StaticType::ident("Record");

        record.push_param(StaticType::ident("string"));
        record.push_param(StaticType::ident("string"));

        let mut u = StaticType::ident("U");
        u.extends(record);
        u.condition(TypeCondition::new(map, StaticType::ident("number")));

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
        let mut ident = StaticType::ident("string");
        ident.or(StaticType::string("foo"));

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

        let mut val = StaticType::ident("null");
        val.or(StaticType::ident("boolean"));
        val.or(StaticType::ident("object"));

        record.push_param(key);
        record.push_param(val);

        let mut u = StaticType::ident("U");
        u.extends(record);

        let expected = expect!["U extends Record<string, null | boolean | object>"];

        expect_raw_ts(&u, &expected);
    }

    #[test]
    fn extends_keyof() {
        let mut blog_node = StaticType::ident("BlogNode");
        blog_node.keyof();

        let mut p = StaticType::ident("P");
        p.extends(blog_node);

        let expected = expect!["P extends keyof BlogNode"];

        expect_raw_ts(&p, &expected);
    }

    #[test]
    fn type_ident_with_extends_condition() {
        let mut record = StaticType::ident("Record");

        record.push_param(StaticType::ident("string"));

        let mut u = StaticType::ident("U");
        u.extends(record);
        u.condition(TypeCondition::new(
            StaticType::ident("string"),
            StaticType::ident("number"),
        ));

        let expected = expect!["U extends Record<string> ? string : number"];

        expect_raw_ts(&u, &expected);
    }

    #[test]
    fn simple_type_definition() {
        let mut asc = StaticType::string("ASC");
        asc.or(StaticType::string("DESC"));

        let r#type = Type::new(StaticType::ident("OrderByDirection"), asc);

        let expected = expect![[r#"
            type OrderByDirection = 'ASC' | 'DESC'
        "#]];

        expect_ts(&r#type, &expected);
    }

    #[test]
    fn export_type_definition() {
        let mut desc = StaticType::string("ASC");
        desc.or(StaticType::string("DESC"));

        let r#type = Type::new(StaticType::ident("OrderByDirection"), desc);

        let r#type = Export::new(r#type);

        let expected = expect![[r#"export type OrderByDirection = 'ASC' | 'DESC'"#]];
        expected.assert_eq(&r#type.to_string());
    }
}
