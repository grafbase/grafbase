use super::{ExtensionInputValueError, FieldSetError, InputValueError};

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("Could not parse GraphQL schema: {0}")]
    GraphQLSchemaParsingError(#[from] cynic_parser::Error),
    #[error("Invalid GraphQL schema: {0}")]
    GraphQLSchemaValidationError(String),
    #[error("Invalid URL '{url}': {err}")]
    InvalidUrl { url: String, err: String },
    #[error("At {} for the extension '{}' directive @{}: {}", .0.location, .0.id, .0.directive, .0.err)]
    ExtensionDirectiveArgumentsError(Box<ExtensionDirectiveArgumentsError>),
    #[error("At {location}, a required field argument is invalid: {err}")]
    RequiredFieldArgumentCoercionError { location: String, err: InputValueError },
    #[error("At {}, encountered an invalid FieldSet: {}", .0.location, .0.err)]
    InvalidFieldSet(Box<InvalidFieldSetError>),
    #[error("An input value named '{name}' has an invalid default value: {err}")]
    DefaultValueCoercionError { name: String, err: InputValueError },
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

impl From<ExtensionDirectiveArgumentsError> for BuildError {
    fn from(err: ExtensionDirectiveArgumentsError) -> Self {
        BuildError::ExtensionDirectiveArgumentsError(Box::new(err))
    }
}

#[derive(Debug)]
pub struct ExtensionDirectiveLocationError {
    pub id: extension_catalog::Id,
    pub directive: String,
    pub location: &'static str,
    pub expected: Vec<&'static str>,
}

impl From<ExtensionDirectiveLocationError> for BuildError {
    fn from(err: ExtensionDirectiveLocationError) -> Self {
        BuildError::ExtensionDirectiveLocationError(Box::new(err))
    }
}

#[derive(Debug)]
pub struct SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError {
    pub id: extension_catalog::Id,
    pub subgraph: String,
    pub other_id: extension_catalog::Id,
}

impl From<SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError> for BuildError {
    fn from(err: SelectionSetResolverExtensionCannotBeMixedWithOtherResolversError) -> Self {
        BuildError::SelectionSetResolverExtensionCannotBeMixedWithOtherResolvers(Box::new(err))
    }
}

#[derive(Debug)]
pub struct InvalidFieldSetError {
    pub location: String,
    pub err: FieldSetError,
}

impl From<InvalidFieldSetError> for BuildError {
    fn from(err: InvalidFieldSetError) -> Self {
        BuildError::InvalidFieldSet(Box::new(err))
    }
}
