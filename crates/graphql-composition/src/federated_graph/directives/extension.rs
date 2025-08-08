use crate::federated_graph::{ExtensionId, StringId, SubgraphId, Value};

pub const EXTENSION_LINK_ENUM: &str = "extension__Link";
pub const EXTENSION_LINK_DIRECTIVE: &str = "extension__link";
pub const EXTENSION_DIRECTIVE_DIRECTIVE: &str = "extension__directive";

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
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct ExtensionDirective {
    pub subgraph_id: SubgraphId,
    pub extension_id: ExtensionId,
    pub name: StringId,
    pub arguments: Option<Vec<(StringId, Value)>>,
}

/// ```ignore,graphql
/// input extension__LinkSchemaDirective {
///    graph: join__Graph!
///    name: String!
///    arguments: DirectiveArguments
/// }
/// ```
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct ExtensionLinkSchemaDirective {
    pub subgraph_id: SubgraphId,
    pub name: StringId,
    pub arguments: Option<Vec<(StringId, Value)>>,
}
