use std::{
    borrow::Cow,
    fmt::{self, Write},
};

#[derive(Debug)]
pub struct Quoted(Cow<'static, str>);

#[derive(Debug)]
pub struct Identifier(Cow<'static, str>);

#[derive(Debug)]
pub enum TypeName {
    Ident(Identifier),
    String(Quoted),
}

impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeName::Ident(ref i) => i.fmt(f),
            TypeName::String(ref i) => i.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum TypeKind {
    Static(StaticType),
    Mapped(MappedType),
}

impl From<StaticType> for TypeKind {
    fn from(value: StaticType) -> Self {
        Self::Static(value)
    }
}

impl From<MappedType> for TypeKind {
    fn from(value: MappedType) -> Self {
        Self::Mapped(value)
    }
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Static(s) => s.fmt(f),
            TypeKind::Mapped(m) => m.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct TypeCondition {
    left: TypeKind,
    right: TypeKind,
}

impl TypeCondition {
    #[must_use]
    pub fn new(left: impl Into<TypeKind>, right: impl Into<TypeKind>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl fmt::Display for TypeCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "? {} : {}", self.left, self.right)
    }
}

#[derive(Debug)]
pub struct StaticType {
    name: TypeName,
    params: Vec<StaticType>,
    or: Vec<StaticType>,
    extends: Option<Box<StaticType>>,
    condition: Option<Box<TypeCondition>>,
    keyof: bool,
}

impl StaticType {
    pub fn ident(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: TypeName::Ident(Identifier::new(name)),
            params: Vec::new(),
            or: Vec::new(),
            extends: None,
            condition: None,
            keyof: false,
        }
    }

    pub fn string(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: TypeName::String(Quoted::new(name)),
            params: Vec::new(),
            or: Vec::new(),
            extends: None,
            condition: None,
            keyof: false,
        }
    }

    #[must_use]
    pub fn extends(mut self, ident: StaticType) -> Self {
        self.extends = Some(Box::new(ident));
        self
    }

    #[must_use]
    pub fn or(mut self, ident: StaticType) -> Self {
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

    pub fn push_param(&mut self, param: StaticType) {
        self.params.push(param);
    }
}

impl fmt::Display for StaticType {
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
    All { alias: Cow<'static, str> },
    Set(Vec<Identifier>),
}

impl fmt::Display for ImportItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportItems::All { alias } => write!(f, "* as {alias}"),
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
    pub fn all_as(import_location: impl Into<Cow<'static, str>>, alias: impl Into<Cow<'static, str>>) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::All { alias: alias.into() },
        }
    }

    pub fn items(import_location: impl Into<Cow<'static, str>>, items: &[&'static str]) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::Set(items.iter().map(|i| Identifier::new(*i)).collect()),
        }
    }

    pub fn push_item(&mut self, identifier: Identifier) {
        match self.items {
            ImportItems::All { .. } => self.items = ImportItems::Set(vec![identifier]),
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
    Type(StaticType),
    Object(ObjectTypeDef),
}

impl From<StaticType> for PropertyValue {
    fn from(value: StaticType) -> Self {
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
    identifier: StaticType,
    properties: Vec<Property>,
}

impl Interface {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            identifier: StaticType::ident(name),
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
            writeln!(f, "  {prop};")?;
        }

        f.write_str("};")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Type {
    identifier: StaticType,
    definition: TypeKind,
}

impl Type {
    pub fn new(identifier: StaticType, definition: impl Into<TypeKind>) -> Self {
        Self {
            identifier,
            definition: definition.into(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {}", self.identifier, self.definition)
    }
}

#[derive(Debug)]
pub enum ExportKind {
    Interface(Interface),
    Type(Type),
}

impl From<Interface> for ExportKind {
    fn from(value: Interface) -> Self {
        ExportKind::Interface(value)
    }
}

impl From<Type> for ExportKind {
    fn from(value: Type) -> Self {
        ExportKind::Type(value)
    }
}

impl fmt::Display for ExportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportKind::Interface(i) => i.fmt(f),
            ExportKind::Type(t) => t.fmt(f),
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
    source: StaticType,
}

impl TypeGenerator {
    #[must_use]
    pub fn new(param: Identifier, source: StaticType) -> Self {
        Self { param, source }
    }
}

impl fmt::Display for TypeGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in {}", self.param, self.source)
    }
}

#[derive(Debug)]
pub enum TypeMapSource {
    Generator(TypeGenerator),
    Static(Property),
}

impl From<TypeGenerator> for TypeMapSource {
    fn from(value: TypeGenerator) -> Self {
        Self::Generator(value)
    }
}

impl From<Property> for TypeMapSource {
    fn from(value: Property) -> Self {
        Self::Static(value)
    }
}

impl fmt::Display for TypeMapSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeMapSource::Generator(g) => g.fmt(f),
            TypeMapSource::Static(s) => s.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct MappedType {
    source: TypeMapSource,
    definition: Box<TypeKind>,
}

impl MappedType {
    pub fn new(source: impl Into<TypeMapSource>, definition: impl Into<TypeKind>) -> Self {
        Self {
            source: source.into(),
            definition: Box::new(definition.into()),
        }
    }
}

impl fmt::Display for MappedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ [{}]: {} }}", self.source, self.definition)
    }
}

#[derive(Debug)]
pub enum BlockItem {
    Export(Export),
    Type(Type),
    Interface(Interface),
    Block(Box<Block>),
}

impl fmt::Display for BlockItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockItem::Export(e) => e.fmt(f),
            BlockItem::Type(t) => t.fmt(f),
            BlockItem::Interface(i) => i.fmt(f),
            BlockItem::Block(b) => b.fmt(f),
        }
    }
}

#[derive(Debug, Default)]
pub struct Block {
    contents: Vec<BlockItem>,
}

impl Block {
    #[must_use] pub fn new() -> Self {
        Self::default()
    }

    pub fn push_content(&mut self, content: impl Into<BlockItem>) {
        self.contents.push(content.into())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\n")?;

        for item in &self.contents {
            writeln!(f, "{item}")?;
        }

        f.write_char('}')?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Function {
    name: Cow<'static, str>,
    params: Vec<Property>,
    returns: Option<TypeKind>,
    body: Block,
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;
    use dprint_plugin_typescript::configuration::{
        Configuration, ConfigurationBuilder, QuoteStyle, SemiColons, TrailingCommas,
    };
    use expect_test::{expect, Expect};
    use tempfile::NamedTempFile;

    static TS_CONFIG: OnceLock<Configuration> = OnceLock::new();

    #[track_caller]
    fn expect_ts(result: impl ToString, expected: &Expect) {
        let config = TS_CONFIG.get_or_init(|| {
            ConfigurationBuilder::new()
                .line_width(80)
                .prefer_hanging(true)
                .prefer_single_line(false)
                .trailing_commas(TrailingCommas::Never)
                .quote_style(QuoteStyle::PreferSingle)
                .indent_width(2)
                .semi_colons(SemiColons::Asi)
                .build()
        });

        let tmp = NamedTempFile::new().unwrap();

        let result = dprint_plugin_typescript::format_text(tmp.path(), &result.to_string(), config)
            .unwrap()
            .unwrap();

        expect_raw_ts(result, expected);
    }

    fn expect_raw_ts(result: impl ToString, expected: &Expect) {
        expected.assert_eq(&result.to_string());
    }

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

        let source = TypeGenerator::new(Identifier::new("P"), ident);
        let definition = StaticType::ident("boolean").or(StaticType::ident("Horse"));
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [P in TruthyKeys<S>]: boolean | Horse }"];

        expect_raw_ts(&map, &expected);
    }

    #[test]
    fn keyof_generator_type_map() {
        let ident = StaticType::ident("Type").keyof();
        let source = TypeGenerator::new(Identifier::new("Property"), ident);
        let definition = StaticType::ident("boolean");
        let map = MappedType::new(source, definition);

        let expected = expect!["{ [Property in keyof Type]: boolean }"];

        expect_raw_ts(&map, &expected);
    }

    #[test]
    fn type_map_in_condition() {
        let ident = StaticType::ident("Type").keyof();
        let source = TypeGenerator::new(Identifier::new("Property"), ident);
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

        let gen = TypeGenerator::new(Identifier::new("P"), ident);

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
    fn import_all() {
        let import = Import::all_as("graphql-request", "gql");

        let expected = expect![[r#"
            import * as gql from 'graphql-request'
        "#]];

        expect_ts(import, &expected);
    }

    #[test]
    fn import_one() {
        let import = Import::items("graphql-request", &["gql"]);

        let expected = expect![[r#"
            import gql from 'graphql-request'
        "#]];

        expect_ts(import, &expected);
    }

    #[test]
    fn import_many() {
        let import = Import::items("graphql-request", &["gql", "GraphQLClient"]);

        let expected = expect![[r#"
            import { gql, GraphQLClient } from 'graphql-request'
        "#]];

        expect_ts(import, &expected);
    }

    #[test]
    fn quoted() {
        let quoted = Quoted::new("test");

        let expected = expect![[r#"
            'test'
        "#]];

        expect_ts(quoted, &expected);
    }

    #[test]
    fn identifier() {
        let identifier = Identifier::new("test");

        let expected = expect![[r#"
            test
        "#]];

        expect_ts(identifier, &expected);
    }

    #[test]
    fn simple_interface() {
        let mut interface = Interface::new("BlogNode");
        interface.push_property(Property::new("id", StaticType::ident("string")));
        interface.push_property(Property::new("name", StaticType::ident("string")));
        interface.push_property(Property::new("owner", StaticType::ident("UserNode")));
        interface.push_property(Property::new("createdAt", StaticType::ident("Date")));
        interface.push_property(Property::new("updatedAt", StaticType::ident("Date")).optional());

        let expected = expect![[r#"
            interface BlogNode {
              id: string
              name: string
              owner: number
              createdAt: Date
              updatedAt?: Date
            }"#]];

        expect_ts(&interface, &expected);
    }

    #[test]
    fn simple_type_definition() {
        let r#type = Type::new(
            StaticType::ident("OrderByDirection"),
            StaticType::string("ASC").or(StaticType::string("DESC")),
        );

        let expected = expect![[r#"type OrderByDirection = 'ASC' | 'DESC'"#]];

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

    #[test]
    fn interface_with_nested_object() {
        let mut object = ObjectTypeDef::new();
        object.push_property(Property::new("node", StaticType::ident("BlogSelect")));
        object.push_property(Property::new("age", StaticType::ident("number")));

        let mut interface = Interface::new("BlogCollectionSelect");
        interface.push_property(Property::new("fields", object));
        interface.push_property(Property::new("name", StaticType::ident("string")));

        let expected = expect![[r#"
            interface BlogCollectionSelect {
              fields: { node: BlogSelect, age: number }
              name: string
            }"#]];

        expect_ts(&interface, &expected);
    }

    #[test]
    fn export_interface() {
        let mut interface = Interface::new("User");
        interface.push_property(Property::new("id", StaticType::ident("string")));

        let expected = expect![[r#"
            export interface User {
              id: string
            }"#]];

        expect_ts(&Export::new(interface), &expected);
    }
}
