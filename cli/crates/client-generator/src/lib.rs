use std::{
    borrow::Cow,
    fmt::{self, Write},
};

#[derive(Debug)]
pub struct Quoted(Cow<'static, str>);

#[derive(Debug)]
pub struct Template(Cow<'static, str>);

#[derive(Debug, Clone)]
pub struct Identifier(Cow<'static, str>);

impl From<Cow<'static, str>> for Identifier {
    fn from(value: Cow<'static, str>) -> Self {
        Self(value)
    }
}

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

impl Template {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
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

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.0)
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
pub struct Entry {
    key: Identifier,
    value: Expression,
}

impl Entry {
    pub fn new(key: Identifier, value: impl Into<Expression>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[derive(Debug)]
pub struct Object {
    entries: Vec<Entry>,
}

impl Object {
    #[must_use]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn entry(&mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Expression>) {
        self.entries.push(Entry::new(Identifier::new(key), value))
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{ ")?;

        for (i, entry) in self.entries.iter().enumerate() {
            entry.fmt(f)?;

            if i < self.entries.len() - 1 {
                f.write_str(", ")?;
            }
        }

        f.write_str(" }")?;

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
pub enum BlockItemKind {
    Type(Type),
    Interface(Interface),
    Block(Box<Block>),
    Expression(Expression),
    Statement(Statement),
    Newline,
}

#[derive(Debug)]
pub struct BlockItem {
    kind: BlockItemKind,
}

impl BlockItem {
    #[must_use]
    pub fn newline() -> Self {
        Self {
            kind: BlockItemKind::Newline,
        }
    }
}

impl From<Type> for BlockItemKind {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

impl From<Interface> for BlockItemKind {
    fn from(value: Interface) -> Self {
        Self::Interface(value)
    }
}

impl From<Block> for BlockItemKind {
    fn from(value: Block) -> Self {
        Self::Block(Box::new(value))
    }
}

impl From<Statement> for BlockItemKind {
    fn from(value: Statement) -> Self {
        Self::Statement(value)
    }
}

impl From<Expression> for BlockItemKind {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}

impl From<Value> for BlockItemKind {
    fn from(value: Value) -> Self {
        Self::Expression(Expression::from(value))
    }
}

impl From<Return> for BlockItemKind {
    fn from(value: Return) -> Self {
        Self::Statement(Statement::from(value))
    }
}

impl<T> From<T> for BlockItem
where
    T: Into<BlockItemKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

impl fmt::Display for BlockItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            BlockItemKind::Type(ref t) => t.fmt(f),
            BlockItemKind::Interface(ref i) => i.fmt(f),
            BlockItemKind::Block(ref b) => b.fmt(f),
            BlockItemKind::Expression(ref e) => e.fmt(f),
            BlockItemKind::Statement(ref s) => s.fmt(f),
            BlockItemKind::Newline => writeln!(f),
        }
    }
}

#[derive(Debug, Default)]
pub struct Block {
    contents: Vec<BlockItem>,
}

impl Block {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, content: impl Into<BlockItem>) {
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

impl Function {
    pub fn new(name: impl Into<Cow<'static, str>>, body: Block) -> Self {
        Self {
            name: name.into(),
            params: Vec::new(),
            returns: None,
            body,
        }
    }

    pub fn returns(mut self, r#type: impl Into<TypeKind>) -> Self {
        self.returns = Some(r#type.into());
        self
    }

    pub fn push_param(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<PropertyValue>) -> Self {
        self.params.push(Property::new(key, value));
        self
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function {}(", self.name)?;

        for param in &self.params {
            write!(f, "{param},")?;
        }

        f.write_char(')')?;

        if let Some(ref returns) = self.returns {
            write!(f, ": {returns} {}", self.body)?;
        } else {
            write!(f, " {}", self.body)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Return {
    expression: Expression,
}

impl Return {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl fmt::Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.expression)
    }
}

#[derive(Debug)]
pub struct TypeOf {
    expression: Expression,
}

impl TypeOf {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl fmt::Display for TypeOf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "typeof {}", self.expression)
    }
}

#[derive(Debug)]
pub enum ValueKind {
    Object(Object),
    Template(Template),
    String(Quoted),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
}

#[derive(Debug)]
pub struct Value {
    kind: ValueKind,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ValueKind::Object(ref obj) => obj.fmt(f),
            ValueKind::String(ref s) => s.fmt(f),
            ValueKind::Template(ref s) => s.fmt(f),
            ValueKind::Number(n) => {
                if n.is_nan() {
                    f.write_str("NaN")
                } else if n.is_infinite() && n.is_sign_positive() {
                    f.write_str("Infinity")
                } else if n.is_infinite() {
                    f.write_str("-Infinity")
                } else if n == n.trunc() {
                    write!(f, "{n:.0}")
                } else {
                    n.fmt(f)
                }
            }
            ValueKind::Boolean(b) => b.fmt(f),
            ValueKind::Null => f.write_str("null"),
            ValueKind::Undefined => f.write_str("undefined"),
        }
    }
}

impl<T> From<T> for Value
where
    T: Into<ValueKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

impl From<Object> for ValueKind {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<&'static str> for ValueKind {
    fn from(value: &'static str) -> Self {
        Self::String(Quoted::new(value))
    }
}

impl From<Template> for ValueKind {
    fn from(value: Template) -> Self {
        Self::Template(value)
    }
}

impl From<String> for ValueKind {
    fn from(value: String) -> Self {
        Self::String(Quoted::new(value))
    }
}

impl From<isize> for ValueKind {
    fn from(value: isize) -> Self {
        Self::Number(value as f64)
    }
}

impl From<bool> for ValueKind {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for ValueKind {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[derive(Debug)]
pub enum StatementKind {
    Assignment(Assignment),
    Conditional(Conditional),
    Return(Return),
}

impl From<Assignment> for StatementKind {
    fn from(value: Assignment) -> Self {
        Self::Assignment(value)
    }
}

impl From<Conditional> for StatementKind {
    fn from(value: Conditional) -> Self {
        Self::Conditional(value)
    }
}

impl From<Return> for StatementKind {
    fn from(value: Return) -> Self {
        Self::Return(value)
    }
}

#[derive(Debug)]
pub struct Statement {
    kind: StatementKind,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            StatementKind::Assignment(ref v) => v.fmt(f),
            StatementKind::Conditional(ref v) => v.fmt(f),
            StatementKind::Return(ref v) => v.fmt(f),
        }
    }
}

impl<T> From<T> for Statement
where
    T: Into<StatementKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

#[derive(Debug)]
pub enum ExpressionKind {
    Variable(Identifier),
    Value(Value),
    TypeOf(Box<TypeOf>),
    Equals(Box<Equals>),
    Closure(Closure),
}

#[derive(Debug)]
pub struct Expression {
    kind: ExpressionKind,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ExpressionKind::Variable(ref v) => v.fmt(f),
            ExpressionKind::Value(ref v) => v.fmt(f),
            ExpressionKind::TypeOf(ref v) => v.fmt(f),
            ExpressionKind::Equals(ref v) => v.fmt(f),
            ExpressionKind::Closure(ref v) => v.fmt(f),
        }
    }
}

impl From<Value> for ExpressionKind {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<Identifier> for ExpressionKind {
    fn from(value: Identifier) -> Self {
        Self::Variable(value)
    }
}

impl From<TypeOf> for ExpressionKind {
    fn from(value: TypeOf) -> Self {
        Self::TypeOf(Box::new(value))
    }
}

impl From<Equals> for ExpressionKind {
    fn from(value: Equals) -> Self {
        Self::Equals(Box::new(value))
    }
}

impl From<Object> for ExpressionKind {
    fn from(value: Object) -> Self {
        Self::Value(Value::from(value))
    }
}

impl From<Template> for ExpressionKind {
    fn from(value: Template) -> Self {
        Self::Value(Value::from(value))
    }
}

impl From<Closure> for ExpressionKind {
    fn from(value: Closure) -> Self {
        Self::Closure(value)
    }
}

impl<T> From<T> for Expression
where
    T: Into<ExpressionKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

#[derive(Debug)]
pub struct Equals {
    left: Expression,
    right: Expression,
    strict: bool,
}

impl Equals {
    pub fn new(left: impl Into<Expression>, right: impl Into<Expression>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
            strict: true,
        }
    }

    #[must_use]
    pub fn non_strict(mut self) -> Self {
        self.strict = false;

        self
    }
}

impl fmt::Display for Equals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = if self.strict { "===" } else { "==" };

        write!(f, "{} {} {}", self.left, op, self.right)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mutability {
    Existing,
    Const,
    Var,
    Let,
}

impl fmt::Display for Mutability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Existing => Ok(()),
            Self::Const => f.write_str("const "),
            Self::Var => f.write_str("var "),
            Self::Let => f.write_str("let "),
        }
    }
}

#[derive(Debug)]
pub struct Assignment {
    left: Identifier,
    right: Expression,
    mutability: Mutability,
    r#type: Option<TypeKind>,
}

impl Assignment {
    pub fn new(left: impl Into<Cow<'static, str>>, right: impl Into<Expression>) -> Self {
        Self {
            left: Identifier::new(left),
            right: right.into(),
            mutability: Mutability::Existing,
            r#type: None,
        }
    }

    #[must_use]
    pub fn r#const(mut self) -> Self {
        self.mutability = Mutability::Const;
        self
    }

    #[must_use]
    pub fn var(mut self) -> Self {
        self.mutability = Mutability::Var;
        self
    }

    #[must_use]
    pub fn r#let(mut self) -> Self {
        self.mutability = Mutability::Let;
        self
    }

    pub fn r#type(mut self, typedef: impl Into<TypeKind>) -> Self {
        self.r#type = Some(typedef.into());
        self
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.mutability, self.left)?;

        if let Some(ref typedef) = self.r#type {
            write!(f, ": {typedef}")?;
        }

        write!(f, " = {}", self.right)
    }
}

#[derive(Debug)]
pub struct Conditional {
    first_branch: (Expression, Block),
    branches: Vec<(Option<Expression>, Block)>,
}

impl Conditional {
    pub fn new(expr: impl Into<Expression>, block: impl Into<Block>) -> Self {
        Self {
            branches: Vec::new(),
            first_branch: (expr.into(), block.into()),
        }
    }

    pub fn else_if(&mut self, expr: impl Into<Expression>, block: impl Into<Block>) {
        self.branches.push((Some(expr.into()), block.into()));
    }

    pub fn r#else(&mut self, block: impl Into<Block>) {
        self.branches.push((None, block.into()));
    }
}

impl fmt::Display for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) {}", self.first_branch.0, self.first_branch.1)?;

        for branch in &self.branches {
            f.write_str(" else")?;

            if let Some(ref condition) = branch.0 {
                write!(f, " if ({}) {}", condition, branch.1)?;
            } else {
                write!(f, " {}", branch.1)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Closure {
    params: Vec<Identifier>,
    input_types: Vec<TypeKind>,
    return_type: Option<TypeKind>,
    body: Block,
}

impl Closure {
    #[must_use]
    pub fn new(body: Block) -> Self {
        Self {
            body,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn params(mut self, params: Vec<Identifier>) -> Self {
        self.params = params;
        self
    }

    pub fn returns(mut self, return_type: impl Into<TypeKind>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    #[must_use]
    pub fn typed_params(mut self, params: Vec<(Identifier, impl Into<TypeKind>)>) -> Self {
        let (params, input_types): (Vec<_>, Vec<_>) = params.into_iter().map(|(a, b)| (a, b.into())).unzip();

        self.params = params;
        self.input_types = input_types;

        self
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('(')?;

        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{param}")?;

            if !self.input_types.is_empty() {
                write!(f, ": {}", self.input_types[i])?;
            }

            f.write_str(", ")?;
        }

        f.write_char(')')?;

        if let Some(ref return_type) = self.return_type {
            write!(f, ": {return_type}")?;
        }

        write!(f, " => {}", self.body)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::OnceLock};

    use super::*;
    use dprint_plugin_typescript::configuration::{
        Configuration, ConfigurationBuilder, QuoteStyle, SemiColons, TrailingCommas,
    };
    use expect_test::{expect, Expect};
    use indoc::indoc;

    #[track_caller]
    fn expect_ts(result: impl ToString, expected: &Expect) {
        static TS_CONFIG: OnceLock<Configuration> = OnceLock::new();

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

        let result = dprint_plugin_typescript::format_text(&PathBuf::from("test.ts"), &result.to_string(), config)
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
              owner: UserNode
              createdAt: Date
              updatedAt?: Date
            }
        "#]];

        expect_ts(&interface, &expected);
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
              fields: { node: BlogSelect; age: number }
              name: string
            }
        "#]];

        expect_ts(&interface, &expected);
    }

    #[test]
    fn export_interface() {
        let mut interface = Interface::new("User");
        interface.push_property(Property::new("id", StaticType::ident("string")));

        let expected = expect![[r#"
            export interface User {
              id: string
            }
        "#]];

        expect_ts(&Export::new(interface), &expected);
    }

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

    #[test]
    fn strict_equals() {
        let eq = Equals::new(TypeOf::new(Identifier::new("val")), Value::from("object"));

        let expected = expect![[r#"
            typeof val === 'object'
        "#]];

        expect_ts(&eq, &expected);
    }

    #[test]
    fn non_strict_equals() {
        let eq = Equals::new(Identifier::new("val"), Value::from(true)).non_strict();

        let expected = expect![[r#"
            val == true
        "#]];

        expect_ts(&eq, &expected);
    }

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
    fn template_string() {
        let template = Template::new(indoc! {r#"
            This here is a long template with ${variable} definition.

            We can add newlines, and they are indented nicely.
        "#});

        let assignment = Assignment::new("text", template).r#const();

        let expected = expect![[r#"
            const text = `This here is a long template with ${variable} definition.

            We can add newlines, and they are indented nicely.
            `
        "#]];

        expect_ts(&assignment, &expected);
    }

    #[test]
    fn empty_closure() {
        let closure = Closure::new(Default::default());

        let expected = expect![[r#"
            const fun = () => {
            }
        "#]];

        let assignment = Assignment::new("fun", closure).r#const();

        expect_ts(&assignment, &expected);
    }

    #[test]
    fn closure_with_param() {
        let identifier = Identifier::new("a");

        let mut body = Block::new();
        body.push(Return::new(identifier.clone()));

        let closure = Closure::new(body).params(vec![identifier]);

        let expected = expect![[r#"
            const fun = (a) => {
              return a
            }
        "#]];

        let assignment = Assignment::new("fun", closure).r#const();

        expect_ts(&assignment, &expected);
    }

    #[test]
    fn closure_with_params() {
        let a = Identifier::new("a");
        let b = Identifier::new("b");

        let mut body = Block::new();
        body.push(Return::new(Equals::new(a.clone(), b.clone())));

        let closure = Closure::new(body)
            .params(vec![a, b])
            .returns(StaticType::ident("boolean"));

        let expected = expect![[r#"
            const fun = (a, b): boolean => {
              return a === b
            }
        "#]];

        let assignment = Assignment::new("fun", closure).r#const();

        expect_ts(&assignment, &expected);
    }

    #[test]
    fn closure_with_typed_params() {
        let identifier = Identifier::new("a");

        let mut body = Block::new();
        body.push(Return::new(identifier.clone()));

        let closure = Closure::new(body)
            .typed_params(vec![(identifier, StaticType::ident("string"))])
            .returns(StaticType::ident("string"));

        let expected = expect![[r#"
            const fun = (a: string): string => {
              return a
            }
        "#]];

        let assignment = Assignment::new("fun", closure).r#const();

        expect_ts(&assignment, &expected);
    }
}
