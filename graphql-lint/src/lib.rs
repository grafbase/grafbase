use cynic_parser::type_system::{
    Definition, Directive, DirectiveDefinition, EnumDefinition, EnumValueDefinition, FieldDefinition,
    InputObjectDefinition, InputValueDefinition, InterfaceDefinition, ObjectDefinition, ScalarDefinition,
    TypeDefinition, UnionDefinition,
};
use cynic_parser::TypeSystemDocument;
use heck::{ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase};
use thiserror::Error;

enum CaseMatch<'a> {
    Correct,
    Incorrect { current: &'a str, fix: String },
}

enum Case {
    Pascal,
    ShoutySnake,
    Camel,
}

pub enum Severity {
    Warning,
}

#[derive(Error, Debug)]
pub enum LinterError {
    #[error("encountered a parsing error:\n{0}")]
    Parse(String),
}

pub fn lint(schema: &str) -> Result<Vec<(String, Severity)>, LinterError> {
    let parsed_schema =
        cynic_parser::parse_type_system_document(schema).map_err(|error| LinterError::Parse(error.to_string()))?;
    Ok(SchemaLinter::new().lint(&parsed_schema))
}

struct SchemaLinter {
    diagnostics: Vec<(String, Severity)>,
}

impl<'a> SchemaLinter {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn lint(mut self, schema: &'a TypeSystemDocument) -> Vec<(String, Severity)> {
        schema.definitions().for_each(|definition| match definition {
            Definition::Schema(_) => {}
            Definition::SchemaExtension(_) => {}
            // TODO: we can optimize this by not rechecking spelling for extensions.
            // We'll also need to do this to avoid duplicate warnings if extending a type with an incorrect name
            Definition::TypeExtension(r#type) | Definition::Type(r#type) => {
                match r#type {
                    TypeDefinition::Scalar(scalar) => {
                        self.visit_scalar(scalar);
                        scalar
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(r#type, directive));
                    }
                    TypeDefinition::Object(object) => {
                        self.visit_object(object);
                        object
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(r#type, directive));
                        object.fields().for_each(|field| {
                            self.visit_field(r#type, field);
                            field
                                .arguments()
                                .for_each(|argument| self.visit_field_argument(r#type, field, argument));
                            field.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_field(r#type, field, directive_usage);
                            });
                        });
                    }
                    TypeDefinition::Interface(interface) => {
                        self.visit_interface(interface);
                        interface
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(r#type, directive));
                        interface.fields().for_each(|field| {
                            self.visit_field(r#type, field);
                            field
                                .arguments()
                                .for_each(|argument| self.visit_field_argument(r#type, field, argument));
                            field.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_field(r#type, field, directive_usage);
                            });
                        });
                    }
                    TypeDefinition::Union(union) => {
                        self.visit_union(union);
                        union
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(r#type, directive))
                    }
                    TypeDefinition::Enum(r#enum) => {
                        self.visit_enum(r#enum);
                        r#enum
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(r#type, directive));
                        r#enum.values().for_each(|value| {
                            self.visit_enum_value(r#type, value);
                            value.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_enum_value(r#enum, value, directive_usage);
                            });
                        });
                    }
                    TypeDefinition::InputObject(input_object) => {
                        self.visit_input_object(input_object);
                        input_object
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(r#type, directive));
                        input_object.fields().for_each(|input_value| {
                            self.visit_input_value(r#type, input_value);
                            input_value.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_input_value(input_object, input_value, directive_usage);
                            });
                        });
                    }
                };
            }
            Definition::Directive(directive) => {
                self.visit_directive(directive);
                directive
                    .arguments()
                    .for_each(|argument| self.visit_directive_argument(directive, argument));
            }
        });

        self.diagnostics
    }

    fn case_check(current: &'a str, case: Case) -> CaseMatch<'_> {
        let fix = match case {
            Case::Pascal => current.to_pascal_case(),
            Case::ShoutySnake => current.to_shouty_snake_case(),
            Case::Camel => current.to_lower_camel_case(),
        };

        if fix == current {
            CaseMatch::Correct
        } else {
            CaseMatch::Incorrect { current, fix }
        }
    }

    pub fn visit_field_argument(
        &mut self,
        parent_type: TypeDefinition<'_>,
        field: FieldDefinition<'_>,
        argument: InputValueDefinition<'_>,
    ) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(argument.name(), Case::Camel) {
            self.diagnostics.push((
                format!(
                    "argument '{current}' on field '{}' on {} '{}' should be renamed to '{fix}'",
                    field.name(),
                    Self::type_definition_display(parent_type),
                    parent_type.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_argument(&mut self, directive: DirectiveDefinition<'_>, argument: InputValueDefinition<'_>) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(argument.name(), Case::Camel) {
            self.diagnostics.push((
                format!(
                    "argument '{current}' on directive '{}' should be renamed to '{fix}'",
                    directive.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_input_value(&mut self, parent: TypeDefinition<'_>, value: InputValueDefinition<'_>) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(value.name(), Case::Camel) {
            self.diagnostics.push((
                format!(
                    "input value '{current}' on input '{}' should be renamed to '{fix}'",
                    parent.name()
                ),
                Severity::Warning,
            ));
        }
    }

    fn type_definition_display(kind: TypeDefinition<'_>) -> &'static str {
        match kind {
            TypeDefinition::Scalar(_) => "scalar",
            TypeDefinition::Object(_) => "type",
            TypeDefinition::Interface(_) => "interface",
            TypeDefinition::Union(_) => "union",
            TypeDefinition::Enum(_) => "enum",
            TypeDefinition::InputObject(_) => "input",
        }
    }

    pub fn visit_field(&mut self, parent: TypeDefinition<'_>, field: FieldDefinition<'_>) {
        let field_name = field.name();

        // ignore system fields
        if field_name.starts_with("__") {
            return;
        }

        if let CaseMatch::Incorrect { current, fix } = Self::case_check(field_name, Case::Camel) {
            self.diagnostics.push((
                format!(
                    "field '{current}' on {} '{}' should be renamed to '{fix}'",
                    Self::type_definition_display(parent),
                    parent.name()
                ),
                Severity::Warning,
            ));
        }
        match parent.name() {
            "Query" => {
                for prefix in ["query", "get", "list"] {
                    if field_name.starts_with(prefix) {
                        self.diagnostics.push((
                            format!("field '{field_name}' on type 'Query' has a forbidden prefix: '{prefix}'"),
                            Severity::Warning,
                        ));
                        break;
                    }
                }
                if field_name.ends_with("Query") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type 'Query' has a forbidden suffix: 'Query'"),
                        Severity::Warning,
                    ));
                }
            }
            "Mutation" => {
                for prefix in ["mutation", "put", "post", "patch"] {
                    if field_name.starts_with(prefix) {
                        self.diagnostics.push((
                            format!("field '{field_name}' on type 'Mutation' has a forbidden prefix: '{prefix}'"),
                            Severity::Warning,
                        ));
                        break;
                    }
                }
                if field_name.ends_with("Mutation") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type 'Mutation' has a forbidden suffix: 'Mutation'"),
                        Severity::Warning,
                    ));
                }
            }
            "Subscription" => {
                if field_name.starts_with("subscription") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type 'Subscription' has a forbidden prefix: 'subscription'"),
                        Severity::Warning,
                    ));
                }
                if field_name.ends_with("Subscription") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type 'Subscription' has a forbidden suffix: 'Subscription'"),
                        Severity::Warning,
                    ));
                }
            }
            _ => {}
        }
    }

    pub fn visit_directive(&mut self, directive: DirectiveDefinition<'_>) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(directive.name(), Case::Camel) {
            self.diagnostics.push((
                format!("directive '{current}' should be renamed to '{fix}'"),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage(&mut self, parent: TypeDefinition<'_>, directive: Directive<'_>) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of directive 'deprecated' on {} '{}' does not populate the 'reason' argument",
                    Self::type_definition_display(parent),
                    parent.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage_field(
        &mut self,
        parent_type: TypeDefinition<'_>,
        parent_field: FieldDefinition<'_>,
        directive: Directive<'_>,
    ) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of directive 'deprecated' on field '{}' on {} '{}' does not populate the 'reason' argument",
                    parent_field.name(),
                    Self::type_definition_display(parent_type),
                    parent_type.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage_input_value(
        &mut self,
        parent_input: InputObjectDefinition<'_>,
        parent_input_value: InputValueDefinition<'_>,
        directive: Directive<'_>,
    ) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of directive 'deprecated' on input value '{}' on input '{}' does not populate the 'reason' argument",
                    parent_input_value.name(),
                    parent_input.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage_enum_value(
        &mut self,
        parent_enum: EnumDefinition<'_>,
        parent_value: EnumValueDefinition<'_>,
        directive: Directive<'_>,
    ) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of directive 'deprecated' on enum value '{}' on enum '{}' does not populate the 'reason' argument",
                    parent_value.value(),
                    parent_enum.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_input_object(&mut self, _input_object: InputObjectDefinition<'_>) {}

    pub fn visit_union(&mut self, union: UnionDefinition<'_>) {
        let union_name = union.name();
        if union_name.starts_with("Union") {
            self.diagnostics.push((
                format!("union '{union_name}' has a forbidden prefix: 'Union'"),
                Severity::Warning,
            ));
        }
        if union_name.ends_with("Union") {
            self.diagnostics.push((
                format!("union '{union_name}' has a forbidden suffix: 'Union'"),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_scalar(&mut self, _scalar: ScalarDefinition<'_>) {}

    pub fn visit_interface(&mut self, object: InterfaceDefinition<'_>) {
        let interface_name = object.name();
        if interface_name.starts_with("Interface") {
            self.diagnostics.push((
                format!("interface '{interface_name}' has a forbidden prefix: 'Interface'"),
                Severity::Warning,
            ));
        }
        if interface_name.ends_with("Interface") {
            self.diagnostics.push((
                format!("interface '{interface_name}' has a forbidden suffix: 'Interface'"),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_object(&mut self, object: ObjectDefinition<'_>) {
        let object_name = object.name();

        if let CaseMatch::Incorrect { current, fix } = Self::case_check(object_name, Case::Pascal) {
            self.diagnostics.push((
                format!("type '{current}' should be renamed to '{fix}'"),
                Severity::Warning,
            ));
        }
        if object_name.starts_with("Type") {
            self.diagnostics.push((
                format!("type '{object_name}' has a forbidden prefix: 'Type'"),
                Severity::Warning,
            ));
        }
        if object_name.ends_with("Type") {
            self.diagnostics.push((
                format!("type '{object_name}' has a forbidden suffix: 'Type'"),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_enum(&mut self, r#enum: EnumDefinition<'_>) {
        let enum_name = r#enum.name();
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(enum_name, Case::Pascal) {
            self.diagnostics.push((
                format!("enum '{current}' should be renamed to '{fix}'"),
                Severity::Warning,
            ));
        }
        if enum_name.starts_with("Enum") {
            self.diagnostics.push((
                format!("enum '{enum_name}' has a forbidden prefix: 'Enum'"),
                Severity::Warning,
            ));
        }
        if enum_name.ends_with("Enum") {
            self.diagnostics.push((
                format!("enum '{enum_name}' has a forbidden suffix: 'Enum'"),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_enum_value(&mut self, parent: TypeDefinition<'_>, enum_value: EnumValueDefinition<'_>) {
        let enum_name = parent.name();

        let name = enum_value.value();
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(name, Case::ShoutySnake) {
            self.diagnostics.push((
                format!("value '{current}' on enum '{enum_name}' should be renamed to '{fix}'"),
                Severity::Warning,
            ));
        }
    }
}

#[test]
fn linter() {
    use criterion as _;

    let schema = r#"
        directive @WithDeprecatedArgs(
          ARG: String @deprecated(reason: "Use `newArg`")
          newArg: String
        ) on FIELD

        enum Enum_lowercase @deprecated {
          an_enum_member @deprecated
        }

        enum lowercase_Enum {
          an_enum_member @deprecated
        }
        
        type Query {
          __test: String,
          getHello(name: String!): Enum_lowercase!
          queryHello(name: String!): Enum_lowercase!
          listHello(name: String!): Enum_lowercase!
          helloQuery(name: String!): Enum_lowercase!
        }

        type Mutation {
          __test: String,
          putHello(name: String!): Enum_lowercase!
          mutationHello(name: String!): Enum_lowercase!
          postHello(name: String!): Enum_lowercase!
          patchHello(name: String!): Enum_lowercase!
          helloMutation(name: String!): Enum_lowercase!
        }

        type Subscription {
          __test: String,
          subscriptionHello(name: String!): Enum_lowercase!
          helloSubscription(name: String!): Enum_lowercase!
        }

        type TypeTest {
          name: String @deprecated
        }

        type TestType {
           name: string
        }

        type other {
           name: string
        }

        scalar CustomScalar @specifiedBy(url: "https://specs.example.com/rfc1") @deprecated

        union UnionTest @deprecated = testType | typeTest

        union TestUnion = testType | typeTest

        interface GameInterface {
          title: String!
          publisher: String! @deprecated
        }

        interface InterfaceGame @deprecated {
          title: String!
          publisher: String!
        }

        input TEST @deprecated {
          OTHER: String @deprecated
        }

        type hello @deprecated {
          Test(NAME: String): String
        }

        extend type hello {
          GOODBYE: String
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#;

    let diagnostics = lint(schema).unwrap();

    assert!(!diagnostics.is_empty());

    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.0.clone())
        .collect::<Vec<_>>();
    dbg!(&messages);

    [
        "directive 'WithDeprecatedArgs' should be renamed to 'withDeprecatedArgs'",
        "argument 'ARG' on directive 'WithDeprecatedArgs' should be renamed to 'arg'",
        "enum 'Enum_lowercase' should be renamed to 'EnumLowercase'",
        "enum 'Enum_lowercase' has a forbidden prefix: 'Enum'",
        "usage of directive 'deprecated' on enum 'Enum_lowercase' does not populate the 'reason' argument",
        "value 'an_enum_member' on enum 'Enum_lowercase' should be renamed to 'AN_ENUM_MEMBER'",
        "usage of directive 'deprecated' on enum value 'an_enum_member' on enum 'Enum_lowercase' does not populate the 'reason' argument",
        "enum 'lowercase_Enum' should be renamed to 'LowercaseEnum'",
        "enum 'lowercase_Enum' has a forbidden suffix: 'Enum'",
        "value 'an_enum_member' on enum 'lowercase_Enum' should be renamed to 'AN_ENUM_MEMBER'",
        "usage of directive 'deprecated' on enum value 'an_enum_member' on enum 'lowercase_Enum' does not populate the 'reason' argument",
        "field 'getHello' on type 'Query' has a forbidden prefix: 'get'",
        "field 'queryHello' on type 'Query' has a forbidden prefix: 'query'",
        "field 'listHello' on type 'Query' has a forbidden prefix: 'list'",
        "field 'helloQuery' on type 'Query' has a forbidden suffix: 'Query'",
        "field 'putHello' on type 'Mutation' has a forbidden prefix: 'put'",
        "field 'mutationHello' on type 'Mutation' has a forbidden prefix: 'mutation'",
        "field 'postHello' on type 'Mutation' has a forbidden prefix: 'post'",
        "field 'patchHello' on type 'Mutation' has a forbidden prefix: 'patch'",
        "field 'helloMutation' on type 'Mutation' has a forbidden suffix: 'Mutation'",
        "field 'subscriptionHello' on type 'Subscription' has a forbidden prefix: 'subscription'",
        "field 'helloSubscription' on type 'Subscription' has a forbidden suffix: 'Subscription'",
        "type 'TypeTest' has a forbidden prefix: 'Type'",
        "usage of directive 'deprecated' on field 'name' on type 'TypeTest' does not populate the 'reason' argument",
        "type 'TestType' has a forbidden suffix: 'Type'",
        "type 'other' should be renamed to 'Other'",
        "usage of directive 'deprecated' on scalar 'CustomScalar' does not populate the 'reason' argument",
        "union 'UnionTest' has a forbidden prefix: 'Union'",
        "usage of directive 'deprecated' on union 'UnionTest' does not populate the 'reason' argument",
        "union 'TestUnion' has a forbidden suffix: 'Union'",
        "interface 'GameInterface' has a forbidden suffix: 'Interface'",
        "usage of directive 'deprecated' on field 'publisher' on interface 'GameInterface' does not populate the 'reason' argument",
        "interface 'InterfaceGame' has a forbidden prefix: 'Interface'",
        "usage of directive 'deprecated' on interface 'InterfaceGame' does not populate the 'reason' argument",
        "usage of directive 'deprecated' on input 'TEST' does not populate the 'reason' argument",
        "input value 'OTHER' on input 'TEST' should be renamed to 'other'",
        "usage of directive 'deprecated' on input value 'OTHER' on input 'TEST' does not populate the 'reason' argument",
        "type 'hello' should be renamed to 'Hello'",
        "usage of directive 'deprecated' on type 'hello' does not populate the 'reason' argument",
        "field 'Test' on type 'hello' should be renamed to 'test'",
        "argument 'NAME' on field 'Test' on type 'hello' should be renamed to 'name'",
        "type 'hello' should be renamed to 'Hello'",
        "field 'GOODBYE' on type 'hello' should be renamed to 'goodbye'",
    ]
        .iter()
        .for_each(|message| assert!(messages.contains(&message.to_string()), "expected '{message}' to be included in diagnostics"));

    let schema = r#"
        directive @withDeprecatedArgs(
          arg: String @deprecated(reason: "Use `newArg`")
          newArg: String
        ) on FIELD

        enum Lowercase {
          AN_ENUM_MEMBER @deprecated(reason: "")
        }

        type Query {
          __test: String,
          hello(name: String!): Lowercase!
        }

        type Mutation {
          __test: String,
          hello(name: String!): Lowercase!
        }

        type Subscription {
          __test: String,
          hello(name: String!): Lowercase!
        }

        type Test {
          name: String @deprecated(reason: "")
        }

        type Other {
           name: string
        }

        scalar CustomScalar @specifiedBy(url: "https://specs.example.com/rfc1") @deprecated(reason: "")

        union NewTest @deprecated(reason: "") = testType | typeTest

        interface Game @deprecated(reason: "") {
          title: String!
          publisher: String! @deprecated(reason: "")
        }

        input Test @deprecated(reason: "") {
          other: String @deprecated(reason: "")
        }

        type Hello @deprecated(reason: "") {
          test(name: String): String
        }

        extend type Hello {
          goodbye: String
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#;

    let diagnostics = lint(schema).unwrap();

    assert!(diagnostics.is_empty());
}
