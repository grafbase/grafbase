mod directives;
mod wrapping;

pub(crate) use cynic_parser::{
    ConstValue,
    common::{TypeWrappersIter, WrappingType},
    type_system::*,
};
use cynic_parser_deser::ConstDeserializer as _;
use rapidhash::RapidHashMap;

pub(crate) use self::wrapping::*;
use super::BuildError;
pub(crate) use directives::*;

#[derive(Default)]
pub(crate) struct Sdl<'a> {
    pub raw: &'a str,
    pub scalar_count: usize,
    pub enum_count: usize,
    pub union_count: usize,
    pub input_object_count: usize,
    pub object_count: usize,
    pub interface_count: usize,
    pub type_definitions: Vec<TypeDefinition<'a>>,
    pub type_extensions: RapidHashMap<&'a str, Vec<TypeDefinition<'a>>>,
    pub root_types: SdlRootTypes<'a>,
    pub schema_directives: Vec<Directive<'a>>,
    pub subgraphs: RapidHashMap<GraphName<'a>, SdlSubGraph<'a>>,
    pub extensions: RapidHashMap<ExtensionName<'a>, SdlExtension<'a>>,
}

impl std::ops::Index<cynic_parser::Span> for Sdl<'_> {
    type Output = str;

    fn index(&self, span: cynic_parser::Span) -> &Self::Output {
        &self.raw[span.start..span.end]
    }
}

pub(crate) struct SdlExtension<'a> {
    pub url: &'a str,
    pub directives: Vec<ExtensionLinkSchemaDirective<'a>>,
}

pub(crate) struct SdlSubGraph<'a> {
    pub name: Option<&'a str>,
    pub url: Option<&'a str>,
}

#[derive(Default)]
pub(crate) struct SdlRootTypes<'a> {
    pub query: Option<&'a str>,
    pub mutation: Option<&'a str>,
    pub subscription: Option<&'a str>,
}

impl<'a> TryFrom<(&'a str, &'a TypeSystemDocument)> for Sdl<'a> {
    type Error = BuildError;

    fn try_from((raw, doc): (&'a str, &'a TypeSystemDocument)) -> Result<Self, Self::Error> {
        let mut sdl = Sdl {
            raw,
            scalar_count: 0,
            enum_count: 0,
            union_count: 0,
            input_object_count: 0,
            object_count: 0,
            interface_count: 0,
            type_definitions: Vec::new(),
            type_extensions: RapidHashMap::default(),
            root_types: SdlRootTypes::default(),
            schema_directives: Vec::new(),
            subgraphs: RapidHashMap::default(),
            extensions: RapidHashMap::default(),
        };
        let mut schema_definition = None;
        let mut schema_definitions = Vec::new();

        for def in doc.definitions() {
            match def {
                Definition::Schema(def) => {
                    if schema_definition.is_some() {
                        return Err(BuildError::GraphQLSchemaValidationError(
                            "A document must include at most one schema definition".into(),
                        ));
                    }
                    schema_definition = Some(def);
                    schema_definitions.push(def);
                    let last_ix = schema_definitions.len() - 1;
                    schema_definitions.swap(0, last_ix);
                }
                Definition::SchemaExtension(def) => {
                    schema_definitions.push(def);
                }
                Definition::Type(type_definition) => {
                    if let Some(name) = type_definition.name().strip_prefix("join__") {
                        match name {
                            "Graph" => {
                                ingest_join_graph_enum(&mut sdl, type_definition)?;
                            }
                            "FieldSet" => {}
                            _ => {
                                return Err(BuildError::GraphQLSchemaValidationError(format!(
                                    "join__{} is an unknown federation type.",
                                    name
                                )));
                            }
                        }
                    } else if let Some(name) = type_definition.name().strip_prefix("extension__") {
                        match name {
                            "Link" => {
                                ingest_extension_link_enum(&mut sdl, type_definition)?;
                            }
                            _ => {
                                return Err(BuildError::GraphQLSchemaValidationError(format!(
                                    "extension__{} is an unknown extension type.",
                                    name
                                )));
                            }
                        }
                    } else {
                        sdl.type_definitions.push(type_definition);
                        match type_definition {
                            TypeDefinition::Scalar(_) => {
                                sdl.scalar_count += 1;
                            }
                            TypeDefinition::Enum(_) => {
                                sdl.enum_count += 1;
                            }
                            TypeDefinition::Union(_) => {
                                sdl.union_count += 1;
                            }
                            TypeDefinition::InputObject(_) => {
                                sdl.input_object_count += 1;
                            }
                            TypeDefinition::Object(_) => {
                                sdl.object_count += 1;
                            }
                            TypeDefinition::Interface(_) => {
                                sdl.interface_count += 1;
                            }
                        }
                    }
                }
                Definition::TypeExtension(type_definition) => {
                    sdl.type_extensions
                        .entry(type_definition.name())
                        .or_default()
                        .push(type_definition);
                }
                Definition::Directive(directive_definition) => {
                    // Ignoring federation directives which are often included in the document
                    // directly.
                    if !directive_definition.name().starts_with("join__") {
                        tracing::warn!("Directive definitions are ignored.")
                    }
                }
            }
        }

        for schema in schema_definitions {
            sdl.schema_directives.extend(schema.directives());
            for root_type in schema.root_operations() {
                match root_type.operation_type() {
                    cynic_parser::common::OperationType::Query => {
                        if sdl.root_types.query.is_some() {
                            return Err(BuildError::GraphQLSchemaValidationError(
                                "A document must include at most one query root type".into(),
                            ));
                        }
                        sdl.root_types.query = Some(root_type.named_type());
                    }
                    cynic_parser::common::OperationType::Mutation => {
                        if sdl.root_types.mutation.is_some() {
                            return Err(BuildError::GraphQLSchemaValidationError(
                                "A document must include at most one mutation root type".into(),
                            ));
                        }
                        sdl.root_types.mutation = Some(root_type.named_type());
                    }
                    cynic_parser::common::OperationType::Subscription => {
                        if sdl.root_types.subscription.is_some() {
                            return Err(BuildError::GraphQLSchemaValidationError(
                                "A document must include at most one subscription root type".into(),
                            ));
                        }
                        sdl.root_types.subscription = Some(root_type.named_type());
                    }
                }
            }
        }

        finalize(&mut sdl)?;

        Ok(sdl)
    }
}

fn ingest_join_graph_enum<'a>(sdl: &mut Sdl<'a>, ty: TypeDefinition<'a>) -> Result<(), BuildError> {
    let TypeDefinition::Enum(enm) = ty else {
        return Err(BuildError::GraphQLSchemaValidationError(
            "join__Graph must be an enum type".into(),
        ));
    };
    if !sdl.subgraphs.is_empty() {
        return Err(BuildError::GraphQLSchemaValidationError(
            "join__Graph must be defined only once".into(),
        ));
    }
    for value in enm.values() {
        let mut directives = value.directives().filter(|dir| dir.name() == "join__graph");
        if let Some(directive) = directives.next() {
            let dir: JoinGraphDirective<'_> = directive.deserialize().map_err(|err| {
                BuildError::GraphQLSchemaValidationError(format!("Invalid @join__graph directive: {}", err))
            })?;
            sdl.subgraphs.insert(
                GraphName(value.value()),
                SdlSubGraph {
                    name: dir.name,
                    url: dir.url,
                },
            );
            if directives.next().is_some() {
                return Err(BuildError::GraphQLSchemaValidationError(format!(
                    "@join__graph directive may only be applied once multiple times on: {}",
                    &sdl[value.span()]
                )));
            }
        } else {
            sdl.subgraphs
                .insert(GraphName(value.value()), SdlSubGraph { name: None, url: None });
        }
    }
    Ok(())
}

fn ingest_extension_link_enum<'a>(sdl: &mut Sdl<'a>, ty: TypeDefinition<'a>) -> Result<(), BuildError> {
    let TypeDefinition::Enum(enm) = ty else {
        return Err(BuildError::GraphQLSchemaValidationError(
            "extension__Link must be an enum type".into(),
        ));
    };
    if !sdl.extensions.is_empty() {
        return Err(BuildError::GraphQLSchemaValidationError(
            "extension__Link must be defined only once".into(),
        ));
    }
    for value in enm.values() {
        let mut directives = value.directives().filter(|dir| dir.name() == "extension__link");
        let Some(directive) = directives.next() else {
            return Err(BuildError::GraphQLSchemaValidationError(
                "Missing extension__link directive".into(),
            ));
        };
        let dir = directives::parse_extension_link(sdl, directive)?;
        sdl.extensions.insert(
            ExtensionName(value.value()),
            SdlExtension {
                url: dir.url,
                directives: dir.schema_directives,
            },
        );
        if directives.next().is_some() {
            return Err(BuildError::GraphQLSchemaValidationError(format!(
                "@extension__link directive may only be applied once multiple times on: {}",
                &sdl[value.span()]
            )));
        }
    }
    Ok(())
}

fn finalize(sdl: &mut Sdl<'_>) -> Result<(), BuildError> {
    for ext in sdl.extensions.values() {
        for directive in &ext.directives {
            if !sdl.subgraphs.contains_key(&directive.graph) {
                return Err(BuildError::GraphQLSchemaValidationError(format!(
                    "Unknown subgraph {} in extension__link directive",
                    directive.graph
                )));
            }
        }
    }

    Ok(())
}
