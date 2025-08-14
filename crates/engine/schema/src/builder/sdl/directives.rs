use cynic_parser::{ConstValue, Span, type_system as ast};
use cynic_parser_deser::{ConstDeserializer, DeserValue, ValueDeserialize, value::ValueType};

use crate::builder::error::Error;

///```ignore,graphql
/// directive @composite__is(
///     graph: join__Graph!,
///     field: String!
/// ) repeatable on FIELD_DEFINITION | ARGUMENT_DEFINITION
///```
#[derive(Debug)]
pub(crate) struct IsDirective<'a> {
    pub graph: GraphName<'a>,
    pub field: &'a str,
}

pub type RequireDirective<'a> = IsDirective<'a>;

// Cynic doesn't generate a good ValueDeserialize because of the `field` name which it relies upon
// in the generated code and it doesn't respect rename for errors.
impl<'a> ValueDeserialize<'a> for IsDirective<'a> {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        let cynic_parser_deser::DeserValue::Object(obj) = input else {
            return Err(cynic_parser_deser::Error::unexpected_type(
                cynic_parser_deser::value::ValueType::Object,
                input,
            ));
        };
        let mut graph = None;
        let mut graph_span = None;
        let mut field = None;
        let mut field_span = None;
        for obj_field in obj.fields() {
            match obj_field.name() {
                "graph" => {
                    if graph.is_some() {
                        return Err(cynic_parser_deser::Error::DuplicateField {
                            name: "graph".to_string(),
                            original_field_span: graph_span,
                            duplicate_field_span: obj_field.name_span(),
                        });
                    }
                    graph_span = obj_field.name_span();
                    graph = Some(obj_field.value().deserialize()?);
                }
                "field" => {
                    if field.is_some() {
                        return Err(cynic_parser_deser::Error::DuplicateField {
                            name: "field".to_string(),
                            original_field_span: field_span,
                            duplicate_field_span: obj_field.name_span(),
                        });
                    }
                    field_span = obj_field.name_span();
                    field = Some(obj_field.value().deserialize()?);
                }
                other => {
                    return Err(cynic_parser_deser::Error::UnknownField {
                        name: other.to_string(),
                        field_type: obj_field.value().into(),
                        field_span: obj_field.name_span(),
                    });
                }
            }
        }
        let graph = graph.ok_or_else(|| cynic_parser_deser::Error::MissingField {
            name: "graph".to_string(),
            object_span: input.span(),
        })?;
        let field = field.ok_or_else(|| cynic_parser_deser::Error::MissingField {
            name: "field".to_string(),
            object_span: input.span(),
        })?;
        Ok(IsDirective { graph, field })
    }
}

#[derive(ValueDeserialize)]
pub struct CostDirective {
    pub weight: i32,
}

/// ```ignore,graphql
/// directive @listSize(assumedSize: Int, slicingArguments: [String!], sizedFields: [String!], requireOneSlicingArgument: Boolean = true) on FIELD_DEFINITION
/// ```
#[derive(ValueDeserialize)]
pub struct ListSizeDirective<'a> {
    #[deser(rename = "assumedSize")]
    pub assumed_size: Option<u32>,
    // Arguments on the current field to interpret as slice size
    #[deser(default = Vec::new(), rename = "slicingArguments")]
    pub slicing_arguments: Vec<&'a str>,
    // Fields on the child object that this size directive applies to
    #[deser(default = Vec::new(), rename = "sizedFields")]
    pub sized_fields: Vec<&'a str>,
    #[deser(default = true, rename = "requireOneSlicingArgument")]
    pub require_one_slicing_argument: bool,
}

#[derive(ValueDeserialize)]
pub struct DeprecatedDirective<'a> {
    pub reason: Option<&'a str>,
}

pub type DerivedDirective<'a> = LookupDirective<'a>;

///```ignore,graphql
/// directive @composite_lookup(
///     graph: join__Graph!,
/// ) repeatable on FIELD_DEFINITION
///```
#[derive(Debug, ValueDeserialize)]
pub(crate) struct LookupDirective<'a> {
    pub graph: GraphName<'a>,
}

/// ```ignore,graphql
/// directive @join__graph(name: String!, url: String!) on ENUM_VALUE
/// ```
#[derive(Debug, ValueDeserialize)]
pub(crate) struct JoinGraphDirective<'a> {
    pub name: Option<&'a str>,
    pub url: Option<&'a str>,
}

pub(crate) fn as_join_type<'a>(dir: &ast::Directive<'a>) -> Option<Result<(JoinTypeDirective<'a>, Span), Error>> {
    if dir.name() == "join__type" {
        Some(
            dir.deserialize()
                .map(|d| (d, dir.arguments_span()))
                .map_err(|err| (err.to_string(), dir.arguments_span()).into()),
        )
    } else {
        None
    }
}

///```ignore,graphql
/// directive @join__type(
///     graph: join__Graph!,
///     key: join__FieldSet,
///     extension: Boolean! = false,
///     resolvable: Boolean! = true,
///     isInterfaceObject: Boolean! = false
/// ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR
///```
#[derive(Debug, ValueDeserialize)]
pub(crate) struct JoinTypeDirective<'a> {
    pub graph: GraphName<'a>,
    pub key: Option<&'a str>,
    #[expect(unused)]
    #[deser(default = false)]
    pub extension: bool,
    #[deser(default = true)]
    pub resolvable: bool,
    #[deser(default = false, rename = "isInterfaceObject")]
    pub is_interface_object: bool,
}

pub(crate) fn as_join_field<'a>(dir: &ast::Directive<'a>) -> Option<Result<(JoinFieldDirective<'a>, Span), Error>> {
    if dir.name() == "join__field" {
        Some(
            dir.deserialize()
                .map(|d| (d, dir.arguments_span()))
                .map_err(|err| (err.to_string(), dir.arguments_span()).into()),
        )
    } else {
        None
    }
}

///```ignore,graphql
/// directive @join__field(
///     graph: join__Graph,
///     requires: join__FieldSet,
///     provides: join__FieldSet,
///     type: String,
///     external: Boolean,
///     override: String,
///     overrideLabel: String
/// ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION
/// ```
#[derive(Default, Debug, ValueDeserialize)]
pub(crate) struct JoinFieldDirective<'a> {
    pub graph: Option<GraphName<'a>>,
    pub requires: Option<&'a str>,
    pub provides: Option<&'a str>,
    #[deser(rename = "type")]
    pub r#type: Option<&'a str>,
    #[deser(default = false)]
    pub external: bool,
    #[deser(rename = "override")]
    pub r#override: Option<&'a str>,
    #[allow(unused)]
    #[deser(rename = "overrideLabel")]
    pub override_label: Option<OverrideLabel>,
}

#[allow(unused)]
#[derive(Debug)]
pub enum OverrideLabel {
    Percent(u8),
    Unknown(String),
}

impl<'de> ValueDeserialize<'de> for OverrideLabel {
    fn deserialize(input: DeserValue<'de>) -> Result<Self, cynic_parser_deser::Error> {
        let s = input.as_str().ok_or_else(|| {
            cynic_parser_deser::Error::custom("Expected a string for @override(label:)", input.span())
        })?;

        Ok(s.parse().unwrap_or_else(|_| OverrideLabel::Unknown(s.to_owned())))
    }
}

impl std::str::FromStr for OverrideLabel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(percent) = s
            .strip_prefix("percent(")
            .and_then(|suffix| suffix.strip_suffix(')'))
            .and_then(|percent| u8::from_str(percent).ok())
        {
            Ok(OverrideLabel::Percent(percent))
        } else {
            Err(r#"Expected a field of the format "percent(<number>)" "#)
        }
    }
}

pub(crate) fn as_join_implements<'a>(
    dir: &ast::Directive<'a>,
) -> Option<Result<(JoinImplementsDirective<'a>, Span), Error>> {
    if dir.name() == "join__implements" {
        Some(
            dir.deserialize()
                .map(|d| (d, dir.arguments_span()))
                .map_err(|err| (err.to_string(), dir.arguments_span()).into()),
        )
    } else {
        None
    }
}

///```ignore,graphql
/// directive @join__implements(
///     graph: join__Graph!,
///     interface: String!
/// ) repeatable on OBJECT | INTERFACE
/// ```
#[derive(Debug, ValueDeserialize)]
pub(crate) struct JoinImplementsDirective<'a> {
    pub graph: GraphName<'a>,
    pub interface: &'a str,
}

pub(crate) fn as_join_union_member<'a>(
    dir: &ast::Directive<'a>,
) -> Option<Result<(JoinUnionMemberDirective<'a>, Span), Error>> {
    if dir.name() == "join__unionMember" {
        Some(
            dir.deserialize()
                .map(|d| (d, dir.arguments_span()))
                .map_err(|err| (err.to_string(), dir.arguments_span()).into()),
        )
    } else {
        None
    }
}

///```ignore,graphql
/// directive @join__unionMember(
///     graph: join__Graph!,
///     member: String!
/// ) repeatable on UNION
///```
#[derive(Debug, ValueDeserialize)]
pub(crate) struct JoinUnionMemberDirective<'a> {
    pub graph: GraphName<'a>,
    pub member: &'a str,
}

///```ignore,graphql
/// directive @join__enumValue(
///     graph: join__Graph!
/// ) repeatable on ENUM_VALUE
///```
#[allow(unused)]
#[derive(Debug, ValueDeserialize)]
pub(crate) struct JoinEnumValueDirective<'a> {
    pub graph: GraphName<'a>,
}

/// ```ignore,graphl
/// """
/// The directive that associates values of the `extension__Link` enum to the extension's url.
/// """
/// directive @extension__link(
///   """
///   The `@link()`ed extension's url, including name and version.
///   """
///   url: String!
///   """
///   The directives on schema definitions and extensions that are associated with the extension.
///   """
///   schemaDirectives: [extension__LinkSchemaDirective!]
/// ) repeatable on ENUM_VALUE
///
/// ```
#[derive(Debug)]
pub(crate) struct ExtensionLinkDirective<'a> {
    pub url: &'a str,
    pub schema_directives: Vec<(ExtensionLinkSchemaDirective<'a>, Span)>,
}

/// ```ignore,graphql
/// input extension__LinkSchemaDirective {
///    graph: join__Graph!
///    name: String!
///    arguments: DirectiveArguments
/// }
/// ```
#[derive(Debug)]
pub(crate) struct ExtensionLinkSchemaDirective<'a> {
    pub graph: GraphName<'a>,
    pub name: &'a str,
    pub arguments: Option<ConstValue<'a>>,
}

pub(crate) fn parse_extension_link(
    directive: cynic_parser::type_system::Directive<'_>,
) -> Result<ExtensionLinkDirective<'_>, Error> {
    let url = directive
        .arguments()
        .find(|arg| arg.name() == "url")
        .and_then(|arg| arg.value().as_str())
        .ok_or_else(|| {
            (
                "Missing or invalid 'url' argument in @extension__link directive",
                directive.arguments_span(),
            )
        })?;

    let schema_directives = directive
        .arguments()
        .find(|arg| arg.name() == "schemaDirectives")
        .and_then(|arg| arg.value().as_list())
        .map(|directives| {
            directives
                .into_iter()
                .map(|value| {
                    value
                        .as_object()
                        .ok_or_else(|| {
                            (
                                "Expected a schemaDirective object for @extension__link directive",
                                value.span(),
                            )
                        })
                        .and_then(|obj| {
                            let graph = obj
                                .get("graph")
                                .and_then(|arg| arg.as_enum_value())
                                .map(GraphName)
                                .ok_or_else(|| {
                                    (
                                        "Missing or invalid 'graph' argument in schemaDirective for @extension__link directive",
                                        obj.span(),
                                    )
                                })?;

                            let name = obj.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                                (
                                    "Missing or invalid 'name' in schemaDirective for @extension__link directive",
                                    obj.span()
                                )
                            })?;

                            let dir = ExtensionLinkSchemaDirective {
                                graph,
                                name,
                                arguments: obj.get("arguments"),
                            };
                            Ok((dir, value.span()))
                        })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(ExtensionLinkDirective { url, schema_directives })
}

/// ```ignore,graphl
/// """
/// An instance of a directive imported from an extension. The `name` and `arguments` arguments
/// are a hoisted version of the original directive. We do this so we can add the `graph` and
/// `extension` arguments.
/// """
/// directive @extension__directive(
///   "Which subgraph the directive comes from"
///   graph: join__Graph!
///   "Which extension the directive is imported from"
///   extension: grafbase__Extension!
///   "The name of the directive. Composition has removed the import prefix if there was one in the original subgraph schema."
///   name: String!
///   arguments: DirectiveArguments
/// ) repeatable ON FIELD | SCHEMA | SCALAR | OBJECT | FIELD_DEFINITION | ARGUMENT_DEFINITION | INTERFACE | UNION | ENUM | ENUM_VALUE | INPUT_OBJECT | INPUT_FIELD_DEFINITION
/// ```
pub(crate) struct ExtensionDirective<'a> {
    pub graph: GraphName<'a>,
    pub extension: ExtensionName<'a>,
    pub name: &'a str,
    pub arguments: Option<ConstValue<'a>>,
}

pub(crate) fn parse_extension_directive(
    directive: cynic_parser::type_system::Directive<'_>,
) -> Result<ExtensionDirective<'_>, Error> {
    let graph = directive
        .arguments()
        .find(|arg| arg.name() == "graph")
        .and_then(|arg| arg.value().as_enum_value())
        .map(GraphName)
        .ok_or_else(|| {
            (
                "Missing or invalid 'graph' argument in @extension__directive",
                directive.arguments_span(),
            )
        })?;

    let extension = directive
        .arguments()
        .find(|arg| arg.name() == "extension")
        .and_then(|arg| arg.value().as_enum_value())
        .map(ExtensionName)
        .ok_or_else(|| {
            (
                "Missing or invalid 'extension' argument in @extension__directive",
                directive.arguments_span(),
            )
        })?;

    let name = directive
        .arguments()
        .find(|arg| arg.name() == "name")
        .and_then(|arg| arg.value().as_str())
        .ok_or_else(|| {
            (
                "Missing or invalid 'name' argument in @extension__directive",
                directive.arguments_span(),
            )
        })?;

    let arguments = directive
        .arguments()
        .find(|arg| arg.name() == "arguments")
        .map(|arg| arg.value());

    Ok(ExtensionDirective {
        graph,
        extension,
        name,
        arguments,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct GraphName<'a>(pub &'a str);

impl<'a> GraphName<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl std::fmt::Display for GraphName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> ValueDeserialize<'de> for GraphName<'de> {
    fn deserialize(input: DeserValue<'de>) -> Result<Self, cynic_parser_deser::Error> {
        match input {
            DeserValue::Enum(enum_value) => Ok(GraphName(enum_value.name())),
            other => Err(cynic_parser_deser::Error::unexpected_type(ValueType::Enum, other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ExtensionName<'a>(pub &'a str);

impl<'a> ExtensionName<'a> {
    #[allow(unused)]
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl std::fmt::Display for ExtensionName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> ValueDeserialize<'de> for ExtensionName<'de> {
    fn deserialize(input: DeserValue<'de>) -> Result<Self, cynic_parser_deser::Error> {
        match input {
            DeserValue::Enum(enum_value) => Ok(ExtensionName(enum_value.name())),
            other => Err(cynic_parser_deser::Error::unexpected_type(ValueType::Enum, other)),
        }
    }
}
