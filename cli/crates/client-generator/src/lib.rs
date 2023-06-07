use std::{
    borrow::Cow,
    fmt::{self, Write},
};

#[derive(Debug)]
pub struct Quoted(Cow<'static, str>);

#[derive(Debug)]
pub struct Identifier(Cow<'static, str>);

#[derive(Debug)]
pub enum TypeKind {
    Ident(Identifier),
    String(Quoted),
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Ident(ref i) => i.fmt(f),
            TypeKind::String(ref i) => i.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct TypeCondition {
    left: TypeIdentifier,
    right: TypeIdentifier,
}

impl TypeCondition {
    #[must_use]
    pub fn new(left: TypeIdentifier, right: TypeIdentifier) -> Self {
        Self { left, right }
    }
}

impl fmt::Display for TypeCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "? {} : {}", self.left, self.right)
    }
}

#[derive(Debug)]
pub struct TypeIdentifier {
    name: TypeKind,
    params: Vec<TypeIdentifier>,
    or: Vec<TypeIdentifier>,
    extends: Option<Box<TypeIdentifier>>,
    condition: Option<Box<TypeCondition>>,
    keyof: bool,
}

impl TypeIdentifier {
    pub fn ident(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: TypeKind::Ident(Identifier::new(name)),
            params: Vec::new(),
            or: Vec::new(),
            extends: None,
            condition: None,
            keyof: false,
        }
    }

    pub fn string(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: TypeKind::String(Quoted::new(name)),
            params: Vec::new(),
            or: Vec::new(),
            extends: None,
            condition: None,
            keyof: false,
        }
    }

    #[must_use]
    pub fn extends(mut self, ident: TypeIdentifier) -> Self {
        self.extends = Some(Box::new(ident));
        self
    }

    #[must_use]
    pub fn or(mut self, ident: TypeIdentifier) -> Self {
        self.or.push(ident);
        self
    }

    #[must_use]
    pub fn condition(mut self, condition: TypeCondition) -> Self {
        self.condition = Some(Box::new(condition));
        self
    }

    #[must_use]
    pub fn keyof(mut self) -> Self {
        self.keyof = true;
        self
    }

    pub fn push_param(&mut self, param: TypeIdentifier) {
        self.params.push(param);
    }
}

impl fmt::Display for TypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.keyof {
            f.write_str("keyof ")?;
        }

        self.name.fmt(f)?;

        if !self.params.is_empty() {
            f.write_char('<')?;

            for (i, param) in self.params.iter().enumerate() {
                param.fmt(f)?;

                if i < self.params.len() - 1 {
                    f.write_str(", ")?;
                }
            }

            f.write_char('>')?;
        }

        if let Some(ref extends) = self.extends {
            write!(f, " extends {extends}")?;
        }

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

impl Quoted {
    pub(crate) fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }
}

impl Identifier {
    pub(crate) fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }
}

impl fmt::Display for Quoted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum ImportItems {
    All,
    Set(Vec<Identifier>),
}

impl fmt::Display for ImportItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportItems::All => f.write_char('*'),
            ImportItems::Set(ref identifiers) => {
                if identifiers.len() > 1 {
                    f.write_str("{ ")?;
                }

                for (i, ident) in identifiers.iter().enumerate() {
                    ident.fmt(f)?;

                    if i < identifiers.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                if identifiers.len() > 1 {
                    f.write_str(" }")?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct Import {
    items: ImportItems,
    import_location: Quoted,
}

impl Import {
    pub fn new(import_location: impl Into<Cow<'static, str>>) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::All,
        }
    }

    pub fn push_item(&mut self, identifier: Identifier) {
        match self.items {
            ImportItems::All => self.items = ImportItems::Set(vec![identifier]),
            ImportItems::Set(ref mut items) => items.push(identifier),
        }
    }
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "import {} from {}", self.items, self.import_location)
    }
}

#[derive(Debug, Default)]
pub struct ObjectTypeDef {
    properties: Vec<Property>,
    multiline: bool,
}

impl ObjectTypeDef {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn push_property(&mut self, prop: Property) {
        self.properties.push(prop);
    }
}

impl fmt::Display for ObjectTypeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let divider = if self.multiline { "\n" } else { " " };
        let indent = if self.multiline { "  " } else { "" };

        write!(f, "{{{divider}")?;

        for (i, prop) in self.properties.iter().enumerate() {
            write!(f, "{indent}{prop}")?;

            if i < self.properties.len() - 1 {
                write!(f, ",{divider}")?;
            }
        }

        write!(f, "{divider}}}")?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum PropertyValue {
    Type(TypeIdentifier),
    Object(ObjectTypeDef),
}

impl From<TypeIdentifier> for PropertyValue {
    fn from(value: TypeIdentifier) -> Self {
        Self::Type(value)
    }
}

impl From<ObjectTypeDef> for PropertyValue {
    fn from(value: ObjectTypeDef) -> Self {
        Self::Object(value)
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Type(ident) => ident.fmt(f),
            PropertyValue::Object(obj) => obj.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Property {
    key: Cow<'static, str>,
    value: PropertyValue,
    optional: bool,
}

impl Property {
    pub fn new(key: impl Into<Cow<'static, str>>, value: impl Into<PropertyValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            optional: false,
        }
    }

    #[must_use]
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let optional = if self.optional { "?" } else { "" };
        write!(f, "{}{optional}: {}", self.key, self.value)
    }
}

#[derive(Debug)]
pub struct Interface {
    identifier: TypeIdentifier,
    properties: Vec<Property>,
}

impl Interface {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            identifier: TypeIdentifier::ident(name),
            properties: Vec::new(),
        }
    }

    pub fn push_property(&mut self, prop: Property) {
        self.properties.push(prop);
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "interface {} {{", self.identifier)?;

        for prop in &self.properties {
            writeln!(f, "  {prop}")?;
        }

        f.write_char('}')?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum ExportKind {
    Interface(Interface),
}

impl From<Interface> for ExportKind {
    fn from(value: Interface) -> Self {
        ExportKind::Interface(value)
    }
}

impl fmt::Display for ExportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportKind::Interface(i) => i.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Export(ExportKind);

impl Export {
    pub fn new(kind: impl Into<ExportKind>) -> Self {
        Self(kind.into())
    }
}

impl fmt::Display for Export {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "export {}", self.0)
    }
}

#[derive(Debug)]
pub struct TypeGenerator {
    param: Identifier,
    source: TypeIdentifier,
}

impl TypeGenerator {
    #[must_use]
    pub fn new(param: Identifier, source: TypeIdentifier) -> Self {
        Self { param, source }
    }
}

impl fmt::Display for TypeGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in {}", self.param, self.source)
    }
}

#[derive(Debug)]
pub enum TypeSource {
    Generator(TypeGenerator),
    Static(Property),
}

impl From<TypeGenerator> for TypeSource {
    fn from(value: TypeGenerator) -> Self {
        Self::Generator(value)
    }
}

impl From<Property> for TypeSource {
    fn from(value: Property) -> Self {
        Self::Static(value)
    }
}

impl fmt::Display for TypeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeSource::Generator(g) => g.fmt(f),
            TypeSource::Static(s) => s.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct MappedType {
    source: TypeSource,
    definition: TypeIdentifier,
}

impl MappedType {
    pub fn new(source: impl Into<TypeSource>, definition: TypeIdentifier) -> Self {
        Self {
            source: source.into(),
            definition,
        }
    }
}

impl fmt::Display for MappedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ [{}]: {} }}", self.source, self.definition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn property_type_map() {
        let source = Property::new("key", TypeIdentifier::ident("string"));
        let definition = TypeIdentifier::ident("boolean").or(TypeIdentifier::ident("Horse"));
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [key: string]: boolean | Horse }"];

        expected.assert_eq(&map.to_string());
    }

    #[test]
    fn generator_type_map() {
        let mut ident = TypeIdentifier::ident("TruthyKeys");
        ident.push_param(TypeIdentifier::ident("S"));

        let source = TypeGenerator::new(Identifier::new("P"), ident);
        let definition = TypeIdentifier::ident("boolean").or(TypeIdentifier::ident("Horse"));
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [P in TruthyKeys<S>]: boolean | Horse }"];

        expected.assert_eq(&map.to_string());
    }

    #[test]
    fn keyof_generator_type_map() {
        let ident = TypeIdentifier::ident("Type").keyof();
        let source = TypeGenerator::new(Identifier::new("Property"), ident);
        let definition = TypeIdentifier::ident("boolean");
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [Property in keyof Type]: boolean }"];

        expected.assert_eq(&map.to_string());
    }

    #[test]
    fn basic_type_generator() {
        let mut ident = TypeIdentifier::ident("TruthyKeys");
        ident.push_param(TypeIdentifier::ident("S"));

        let gen = TypeGenerator::new(Identifier::new("P"), ident);

        let expected = expect!["P in TruthyKeys<S>"];

        expected.assert_eq(&gen.to_string());
    }

    #[test]
    fn simple_type_ident() {
        let ident = TypeIdentifier::ident("BlogNode");
        let expected = expect!["BlogNode"];

        expected.assert_eq(&ident.to_string());
    }

    #[test]
    fn type_ident_with_or() {
        let ident = TypeIdentifier::ident("string").or(TypeIdentifier::string("foo"));

        let expected = expect!["string | 'foo'"];

        expected.assert_eq(&ident.to_string());
    }

    #[test]
    fn type_ident_with_params() {
        let mut ident = TypeIdentifier::ident("BlogNode");
        ident.push_param(TypeIdentifier::ident("T"));
        ident.push_param(TypeIdentifier::ident("U"));

        let expected = expect!["BlogNode<T, U>"];

        expected.assert_eq(&ident.to_string());
    }

    #[test]
    fn type_ident_with_extends() {
        let mut record = TypeIdentifier::ident("Record");

        let key = TypeIdentifier::ident("string");

        let val = TypeIdentifier::ident("null")
            .or(TypeIdentifier::ident("boolean"))
            .or(TypeIdentifier::ident("object"));

        record.push_param(key);
        record.push_param(val);

        let u = TypeIdentifier::ident("U").extends(record);
        let expected = expect!["U extends Record<string, null | boolean | object>"];

        expected.assert_eq(&u.to_string());
    }

    #[test]
    fn extends_keyof() {
        let blog_node = TypeIdentifier::ident("BlogNode").keyof();
        let u = TypeIdentifier::ident("P").extends(blog_node);

        let expected = expect!["P extends keyof BlogNode"];

        expected.assert_eq(&u.to_string());
    }

    #[test]
    fn type_ident_with_extends_condition() {
        let mut record = TypeIdentifier::ident("Record");

        record.push_param(TypeIdentifier::ident("string"));

        let u = TypeIdentifier::ident("U").extends(record).condition(TypeCondition::new(
            TypeIdentifier::ident("string"),
            TypeIdentifier::ident("number"),
        ));

        let expected = expect!["U extends Record<string> ? string : number"];

        expected.assert_eq(&u.to_string());
    }

    #[test]
    fn import_all() {
        let import = Import::new("graphql-request");
        let expected = expect!["import * from 'graphql-request'"];

        expected.assert_eq(&import.to_string());
    }

    #[test]
    fn import_one() {
        let mut import = Import::new("graphql-request");
        import.push_item(Identifier::new("gql"));

        let expected = expect!["import gql from 'graphql-request'"];
        expected.assert_eq(&import.to_string());
    }

    #[test]
    fn import_many() {
        let mut import = Import::new("graphql-request");
        import.push_item(Identifier::new("gql"));
        import.push_item(Identifier::new("GraphQLClient"));

        let expected = expect!["import { gql, GraphQLClient } from 'graphql-request'"];
        expected.assert_eq(&import.to_string());
    }

    #[test]
    fn quoted() {
        let quoted = Quoted::new("test");
        let expected = expect!["'test'"];

        expected.assert_eq(&quoted.to_string());
    }

    #[test]
    fn identifier() {
        let quoted = Identifier::new("test");
        let expected = expect![[r#"test"#]];

        expected.assert_eq(&quoted.to_string());
    }

    #[test]
    fn simple_interface() {
        let mut interface = Interface::new("BlogNode");
        interface.push_property(Property::new("id", TypeIdentifier::ident("string")));
        interface.push_property(Property::new("name", TypeIdentifier::ident("string")));
        interface.push_property(Property::new("owner", TypeIdentifier::ident("UserNode")));
        interface.push_property(Property::new("createdAt", TypeIdentifier::ident("Date")));
        interface.push_property(Property::new("updatedAt", TypeIdentifier::ident("Date")).optional());

        let expected = expect![[r#"
            interface BlogNode {
              id: string
              name: string
              owner: UserNode
              createdAt: Date
              updatedAt?: Date
            }"#]];

        expected.assert_eq(&interface.to_string());
    }

    #[test]
    fn interface_with_nested_object() {
        let mut object = ObjectTypeDef::new();
        object.push_property(Property::new("node", TypeIdentifier::ident("BlogSelect")));
        object.push_property(Property::new("age", TypeIdentifier::ident("number")));

        let mut interface = Interface::new("BlogCollectionSelect");
        interface.push_property(Property::new("fields", object));
        interface.push_property(Property::new("name", TypeIdentifier::ident("string")));

        let expected = expect![[r#"
            interface BlogCollectionSelect {
              fields: { node: BlogSelect, age: number }
              name: string
            }"#]];

        expected.assert_eq(&interface.to_string());
    }

    #[test]
    fn export_interface() {
        let mut interface = Interface::new("User");
        interface.push_property(Property::new("id", TypeIdentifier::ident("string")));

        let expected = expect![[r#"
            export interface User {
              id: string
            }"#]];

        expected.assert_eq(&Export::new(interface).to_string());
    }
}
