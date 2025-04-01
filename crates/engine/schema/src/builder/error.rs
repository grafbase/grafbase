use super::{
    EntityDefinitionId, EnumDefinitionId, EnumValueId, ExtensionInputValueError, FieldDefinitionId,
    InputObjectDefinitionId, InputValueDefinitionId, InputValueError, InterfaceDefinitionId, ObjectDefinitionId,
    ScalarDefinitionId, UnionDefinitionId, graph::GraphContext,
};

#[derive(Debug, Copy, Clone)]
pub enum SchemaLocation {
    SchemaDirective(federated_graph::SubgraphId),
    Scalar(ScalarDefinitionId, federated_graph::ScalarDefinitionId),
    Object(ObjectDefinitionId, federated_graph::ObjectId),
    Interface(InterfaceDefinitionId, federated_graph::InterfaceId),
    Union(UnionDefinitionId, federated_graph::UnionId),
    Enum(EnumDefinitionId, federated_graph::EnumDefinitionId),
    InputObject(InputObjectDefinitionId, federated_graph::InputObjectId),
    FieldDefinition(FieldDefinitionId, federated_graph::FieldId),
    InputFieldDefinition(
        InputObjectDefinitionId,
        InputValueDefinitionId,
        federated_graph::InputValueDefinitionId,
    ),
    ArgumentDefinition(
        FieldDefinitionId,
        InputValueDefinitionId,
        federated_graph::InputValueDefinitionId,
    ),
    EnumValue(EnumDefinitionId, EnumValueId, federated_graph::EnumValueId),
}

impl SchemaLocation {
    pub fn to_string(self, GraphContext { ctx, graph, .. }: &GraphContext<'_>) -> String {
        match self {
            SchemaLocation::Enum(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::InputObject(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Interface(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Object(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Scalar(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Union(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::FieldDefinition(id, _) => {
                let field = &graph[id];
                let parent_name_id = match field.parent_entity_id {
                    EntityDefinitionId::Interface(id) => graph[id].name_id,
                    EntityDefinitionId::Object(id) => graph[id].name_id,
                };
                format!("{}.{}", ctx.strings[parent_name_id], ctx.strings[field.name_id])
            }
            SchemaLocation::InputFieldDefinition(input_object_id, id, _) => {
                format!(
                    "{}.{}",
                    ctx.strings[graph[input_object_id].name_id], ctx.strings[graph[id].name_id]
                )
            }
            SchemaLocation::ArgumentDefinition(field_id, id, _) => {
                let field = &graph[field_id];
                let parent_name_id = match field.parent_entity_id {
                    EntityDefinitionId::Interface(id) => graph[id].name_id,
                    EntityDefinitionId::Object(id) => graph[id].name_id,
                };
                format!(
                    "{}.{}.{}",
                    ctx.strings[parent_name_id], ctx.strings[field.name_id], ctx.strings[graph[id].name_id]
                )
            }
            SchemaLocation::EnumValue(enum_id, id, _) => {
                format!(
                    "{}.{}",
                    ctx.strings[graph[enum_id].name_id], ctx.strings[graph[id].name_id]
                )
            }
            SchemaLocation::SchemaDirective(id) => {
                format!("subgraph named '{}'", ctx.federated_graph[ctx.federated_graph[id].name])
            }
        }
    }

    pub fn to_cynic_location(self) -> cynic_parser::type_system::DirectiveLocation {
        match self {
            SchemaLocation::Enum(_, _) => cynic_parser::type_system::DirectiveLocation::Enum,
            SchemaLocation::InputObject(_, _) => cynic_parser::type_system::DirectiveLocation::InputObject,
            SchemaLocation::Interface(_, _) => cynic_parser::type_system::DirectiveLocation::Interface,
            SchemaLocation::Object(_, _) => cynic_parser::type_system::DirectiveLocation::Object,
            SchemaLocation::Scalar(_, _) => cynic_parser::type_system::DirectiveLocation::Scalar,
            SchemaLocation::Union(_, _) => cynic_parser::type_system::DirectiveLocation::Union,
            SchemaLocation::FieldDefinition(_, _) => cynic_parser::type_system::DirectiveLocation::FieldDefinition,
            SchemaLocation::EnumValue(_, _, _) => cynic_parser::type_system::DirectiveLocation::EnumValue,
            SchemaLocation::SchemaDirective(_) => cynic_parser::type_system::DirectiveLocation::Schema,
            SchemaLocation::ArgumentDefinition(_, _, _) => {
                cynic_parser::type_system::DirectiveLocation::ArgumentDefinition
            }
            SchemaLocation::InputFieldDefinition(_, _, _) => {
                cynic_parser::type_system::DirectiveLocation::InputFieldDefinition
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("Invalid URL '{url}': {err}")]
    InvalidUrl { url: String, err: String },
    #[error("At {} for the extension '{}' directive @{}: {}", .0.location, .0.id, .0.directive, .0.err)]
    ExtensionDirectiveArgumentsError(Box<ExtensionDirectiveArgumentsError>),
    #[error("At {location}, a required field argument is invalid: {err}")]
    RequiredFieldArgumentCoercionError { location: String, err: InputValueError },
    #[error("An input value named '{name}' has an invalid default value: {err}")]
    DefaultValueCoercionError { name: String, err: InputValueError },
    #[error(transparent)]
    GraphFromSdlError(#[from] federated_graph::DomainError),
    #[error("Unsupported extension: {id}")]
    UnsupportedExtension { id: extension_catalog::Id },
    #[error("Could not load extension at '{url}': {err}")]
    CouldNotLoadExtension { url: String, err: String },
    #[error("Could not parse extension '{id}' GraphQL definitions: {err}")]
    CouldNotParseExtension { id: extension_catalog::Id, err: String },
    #[error("Extension '{id}' does not define any GraphQL definitions, but a directive @{directive} was found")]
    MissingGraphQLDefinitions {
        id: extension_catalog::Id,
        directive: String,
    },
    #[error("Unknown extension directive @{directive} for extension '{id}'")]
    UnknownExtensionDirective {
        id: extension_catalog::Id,
        directive: String,
    },
    #[error("Unknown argument '{argument}' for extension directive @{directive} from '{id}'")]
    UnknownExtensionDirectiveArgument {
        id: extension_catalog::Id,
        directive: String,
        argument: String,
    },
    #[error("Extension {} directive @{} used in the wrong location {}, expected one of: {}", .0.id, .0.directive, .0.location, .0.expected.join(","))]
    ExtensionDirectiveLocationError(Box<ExtensionDirectiveLocationError>),
    #[error("Could not read a @link directive used in the extension {id} GraphQL definitions: {err}")]
    ExtensionCouldNotReadLink { id: extension_catalog::Id, err: String },
    #[error("Extension {id} imports an unknown Grafbase definition: '{name}'")]
    ExtensionLinksToUnknownGrafbaseDefinition { id: extension_catalog::Id, name: String },
    #[error(
        "Resolver extension {id}' directive '{directive}' can only be used on virtual graphs, '{subgraph}' isn't one."
    )]
    ResolverExtensionOnNonVirtualGraph {
        id: extension_catalog::Id,
        directive: String,
        subgraph: String,
    },
    #[error(
        "Selection Set Resolver extension {} cannot be mixed with other resolvers in subgraph '{}', found {}",
        .0.id,
        .0.subgraph,
        .0.other_id
    )]
    SelectionSetResolverExtensionCannotBeMixedWithOtherResolvers(
        Box<SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError>,
    ),
}

#[derive(Debug)]
pub struct ExtensionDirectiveArgumentsError {
    pub id: extension_catalog::Id,
    pub directive: String,
    pub location: String,
    pub err: ExtensionInputValueError,
}

#[derive(Debug)]
pub struct ExtensionDirectiveLocationError {
    pub id: extension_catalog::Id,
    pub directive: String,
    pub location: &'static str,
    pub expected: Vec<&'static str>,
}

#[derive(Debug)]
pub struct SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError {
    pub id: extension_catalog::Id,
    pub subgraph: String,
    pub other_id: extension_catalog::Id,
}
