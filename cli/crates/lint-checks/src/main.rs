use cynic_parser::type_system::{
    Definition, Directive, DirectiveDefinition, EnumDefinition, EnumValueDefinition, FieldDefinition,
    InputObjectDefinition, InputValueDefinition, InterfaceDefinition, ObjectDefinition, ScalarDefinition,
    TypeDefinition, UnionDefinition,
};
use cynic_parser::TypeSystemDocument;
use heck::{ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase};
use std::process::exit;
use std::time::Instant;

fn main() {
    let schema = r#"
        directive @WithDeprecatedArgs(
          ARG: String @deprecated(reason: "Use `newArg`")
          newArg: String
        ) on FIELD

        enum Enum_lowercase {
          an_enum_member @deprecated
        }
        
        type Query {
          __test: String,
          getHello(name: String!): Enum_lowercase!
        }
        
        input TEST {
          OTHER: String
        }

        type hello @deprecated {
          Test(NAME: String): String
        }

        extend type hello {
          GOODBYE: String
        }

        schema {
          query: Query
        }
    "#;
    let diagnostics = run_lint_checks(schema);
    if diagnostics.is_empty() {
        println!("✅ Your schema is perfect. Good job!");
        exit(0);
    }
    diagnostics.iter().for_each(|diagnostic| {
        println!("⚠️ Warning: {}", diagnostic.0);
    });
    exit(1);
}

enum CaseMatch<'a> {
    Correct,
    Incorrect { current: &'a str, fix: String },
}

enum Case {
    Pascal,
    ShoutySnake,
    Camel,
}

enum Severity {
    Warning,
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
                match &r#type {
                    TypeDefinition::Scalar(scalar) => {
                        self.visit_scalar(scalar);
                        scalar
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(&r#type, &directive));
                    }
                    TypeDefinition::Object(object) => {
                        self.visit_object(object);
                        object
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(&r#type, &directive));
                        object.fields().for_each(|field| {
                            self.visit_field(&r#type, &field);
                            field
                                .arguments()
                                .for_each(|argument| self.visit_field_argument(&r#type, &field, &argument));
                            field.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_field(&r#type, &field, &directive_usage);
                            });
                        });
                    }
                    TypeDefinition::Interface(interface) => {
                        self.visit_interface(interface);
                        interface
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(&r#type, &directive));
                        interface.fields().for_each(|field| {
                            self.visit_field(&r#type, &field);
                            field
                                .arguments()
                                .for_each(|argument| self.visit_field_argument(&r#type, &field, &argument));
                            field.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_field(&r#type, &field, &directive_usage);
                            });
                        });
                    }
                    TypeDefinition::Union(union) => {
                        self.visit_union(union);
                        union
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(&r#type, &directive))
                    }
                    TypeDefinition::Enum(r#enum) => {
                        self.visit_enum(r#enum);
                        r#enum
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(&r#type, &directive));
                        r#enum.values().for_each(|value| {
                            self.visit_enum_value(&r#type, &value);
                            value.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_enum_value(r#enum, &value, &directive_usage);
                            });
                        });
                    }
                    TypeDefinition::InputObject(input_object) => {
                        self.visit_input_object(input_object);
                        input_object
                            .directives()
                            .for_each(|directive| self.visit_directive_usage(&r#type, &directive));
                        input_object.fields().for_each(|input_value| {
                            self.visit_input_value(&r#type, &input_value);
                            input_value.directives().for_each(|directive_usage| {
                                self.visit_directive_usage_input_value(input_object, &input_value, &directive_usage);
                            });
                        });
                    }
                };
            }
            Definition::Directive(directive) => {
                self.visit_directive(&directive);
                directive
                    .arguments()
                    .for_each(|argument| self.visit_directive_argument(&directive, &argument));
            }
        });

        self.diagnostics
    }

    fn case_check<T: AsRef<str>>(value: &T, case: Case) -> CaseMatch<'_> {
        let current = value.as_ref();

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
        parent_type: &TypeDefinition<'a>,
        field: &FieldDefinition<'a>,
        argument: &InputValueDefinition<'a>,
    ) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&argument.name(), Case::Camel) {
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

    pub fn visit_directive_argument(
        &mut self,
        directive: &DirectiveDefinition<'a>,
        argument: &InputValueDefinition<'a>,
    ) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&argument.name(), Case::Camel) {
            self.diagnostics.push((
                format!(
                    "argument '{current}' on directive '{}' should be renamed to '{fix}'",
                    directive.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_input_value(&mut self, parent: &TypeDefinition<'a>, value: &InputValueDefinition<'a>) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&value.name(), Case::Camel) {
            self.diagnostics.push((
                format!(
                    "input value '{current}' on input '{}' should be renamed to '{fix}'",
                    parent.name()
                ),
                Severity::Warning,
            ));
        }
    }

    fn type_definition_display(kind: &TypeDefinition<'a>) -> &'static str {
        match kind {
            TypeDefinition::Scalar(_) => "scalar",
            TypeDefinition::Object(_) => "type",
            TypeDefinition::Interface(_) => "interface",
            TypeDefinition::Union(_) => "union",
            TypeDefinition::Enum(_) => "enum",
            TypeDefinition::InputObject(_) => "input",
        }
    }

    pub fn visit_field(&mut self, parent: &TypeDefinition<'a>, field: &FieldDefinition<'a>) {
        let field_name = field.name();

        // ignore system fields
        if field_name.starts_with("__") {
            return;
        }

        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&field_name, Case::Camel) {
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
                            format!("field '{field_name}' on type Query has a forbidden prefix: '{prefix}'"),
                            Severity::Warning,
                        ));
                    }
                }
                if field_name.ends_with("Query") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type Query has a forbidden suffix: 'Query'"),
                        Severity::Warning,
                    ));
                }
            }
            "Mutation" => {
                for prefix in ["mutation", "put", "post", "patch"] {
                    if field_name.starts_with(prefix) {
                        self.diagnostics.push((
                            format!("field '{field_name}' on type Mutation has a forbidden prefix: '{prefix}'"),
                            Severity::Warning,
                        ));
                    }
                }
                if field_name.ends_with("Mutation") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type Mutation has a forbidden suffix: 'Mutation'"),
                        Severity::Warning,
                    ));
                }
            }
            "Subscription" => {
                if field_name.starts_with("subscription") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type Subscription has a forbidden prefix: 'subscription'"),
                        Severity::Warning,
                    ));
                }
                if field_name.ends_with("Subscription") {
                    self.diagnostics.push((
                        format!("field '{field_name}' on type Subscription has a forbidden suffix: 'Subscription'"),
                        Severity::Warning,
                    ));
                }
            }
            _ => {}
        }
    }

    pub fn visit_directive(&mut self, directive: &DirectiveDefinition<'a>) {
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&directive.name(), Case::Camel) {
            self.diagnostics.push((
                format!("directive '{current}' should be renamed to '{fix}'"),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage(&mut self, parent: &TypeDefinition<'a>, directive: &Directive<'a>) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of '@deprecated' on {} '{}' does not populate the 'reason' argument",
                    Self::type_definition_display(parent),
                    parent.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage_field(
        &mut self,
        parent_type: &TypeDefinition<'a>,
        parent_field: &FieldDefinition<'a>,
        directive: &Directive<'a>,
    ) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of '@deprecated' on field '{}' on {} '{}' does not populate the 'reason' argument",
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
        parent_input: &InputObjectDefinition<'a>,
        parent_input_value: &InputValueDefinition<'a>,
        directive: &Directive<'a>,
    ) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of '@deprecated' on input_value '{}' on input '{}' does not populate the 'reason' argument",
                    parent_input_value.name(),
                    parent_input.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_directive_usage_enum_value(
        &mut self,
        parent_enum: &EnumDefinition<'a>,
        parent_value: &EnumValueDefinition<'a>,
        directive: &Directive<'a>,
    ) {
        if directive.name() == "deprecated" && !directive.arguments().any(|argument| argument.name() == "reason") {
            self.diagnostics.push((
                format!(
                    "usage of '@deprecated' on enum value '{}' on enum '{}' does not populate the 'reason' argument",
                    parent_value.value(),
                    parent_enum.name()
                ),
                Severity::Warning,
            ));
        }
    }

    pub fn visit_input_object(&mut self, _input_object: &InputObjectDefinition<'a>) {}

    pub fn visit_union(&mut self, union: &UnionDefinition<'a>) {
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

    pub fn visit_scalar(&mut self, _scalar: &ScalarDefinition<'a>) {}

    pub fn visit_interface(&mut self, object: &InterfaceDefinition<'a>) {
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

    pub fn visit_object(&mut self, object: &ObjectDefinition<'a>) {
        let object_name = object.name();

        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&object_name, Case::Pascal) {
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

    pub fn visit_enum(&mut self, r#enum: &EnumDefinition<'a>) {
        let enum_name = r#enum.name();
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&enum_name, Case::Pascal) {
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

    pub fn visit_enum_value(&mut self, parent: &TypeDefinition<'a>, enum_value: &EnumValueDefinition<'a>) {
        let enum_name = parent.name();

        let name = enum_value.value();
        if let CaseMatch::Incorrect { current, fix } = Self::case_check(&name, Case::ShoutySnake) {
            self.diagnostics.push((
                format!("value '{current}' on enum '{enum_name}' should be renamed to '{fix}'"),
                Severity::Warning,
            ));
        }
    }
}

fn run_lint_checks(schema: &str) -> Vec<(String, Severity)> {
    let start = Instant::now();
    let parsed_schema = cynic_parser::parse_type_system_document(schema).unwrap();
    let diagnostics = SchemaLinter::new().lint(&parsed_schema);
    let end = start.elapsed();
    println!("{end:#?}");
    diagnostics
}
