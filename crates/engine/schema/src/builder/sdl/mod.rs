mod definitions;
mod directives;
mod span;
mod wrapping;

pub(crate) use cynic_parser::{
    ConstValue,
    common::{TypeWrappersIter, WrappingType},
    type_system::*,
};
use cynic_parser_deser::ConstDeserializer as _;
use rapidhash::RapidHashMap;

pub(crate) use self::wrapping::*;
pub(crate) use definitions::*;
pub(crate) use directives::*;
pub(crate) use span::*;

use super::error::Error;

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
    pub url: url::Url,
    pub directives: Vec<(ExtensionLinkSchemaDirective<'a>, Span)>,
}

pub(crate) struct SdlSubGraph<'a> {
    pub name: Option<&'a str>,
    pub url: Option<url::Url>,
}

#[derive(Default)]
pub(crate) struct SdlRootTypes<'a> {
    pub query: Option<&'a str>,
    pub mutation: Option<&'a str>,
    pub subscription: Option<&'a str>,
}

impl<'a> TryFrom<(&'a str, &'a TypeSystemDocument)> for Sdl<'a> {
    type Error = Error;

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
                        return Err(("A document must include at most one schema definition", def.span()).into());
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
                                return Err((
                                    format!("join__{name} is an unknown federation type."),
                                    type_definition.span(),
                                )
                                    .into());
                            }
                        }
                    } else if let Some(name) = type_definition.name().strip_prefix("extension__") {
                        match name {
                            "Link" => {
                                ingest_extension_link_enum(&mut sdl, type_definition)?;
                            }
                            _ => {
                                return Err((
                                    format!("extension__{name} is an unknown extension type.",),
                                    type_definition.span(),
                                )
                                    .into());
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
                            return Err(
                                ("A document must include at most one query root type", root_type.span()).into(),
                            );
                        }
                        sdl.root_types.query = Some(root_type.named_type());
                    }
                    cynic_parser::common::OperationType::Mutation => {
                        if sdl.root_types.mutation.is_some() {
                            return Err((
                                "A document must include at most one mutation root type",
                                root_type.span(),
                            )
                                .into());
                        }
                        sdl.root_types.mutation = Some(root_type.named_type());
                    }
                    cynic_parser::common::OperationType::Subscription => {
                        if sdl.root_types.subscription.is_some() {
                            return Err((
                                "A document must include at most one subscription root type",
                                root_type.span(),
                            )
                                .into());
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

fn ingest_join_graph_enum<'a>(sdl: &mut Sdl<'a>, ty: TypeDefinition<'a>) -> Result<(), Error> {
    let TypeDefinition::Enum(enm) = ty else {
        return Err(("join__Graph must be an enum type", ty.span()).into());
    };
    if !sdl.subgraphs.is_empty() {
        return Err(("join__Graph must be defined only once", enm.span()).into());
    }
    for value in enm.values() {
        let mut directives = value.directives().filter(|dir| dir.name() == "join__graph");
        if let Some(directive) = directives.next() {
            let dir: JoinGraphDirective<'_> = directive.deserialize().map_err(|err| {
                (
                    format!("Invalid @join__graph directive on subgraph {}: {err}", value.value()),
                    directive.arguments_span(),
                )
            })?;
            let url = dir
                .url
                .map(|url| {
                    url::Url::parse(url).map_err(|err| {
                        (
                            format!("Invalid url on subgraph {}: {err}", value.value()),
                            directive.arguments_span(),
                        )
                    })
                })
                .transpose()?;
            sdl.subgraphs
                .insert(GraphName(value.value()), SdlSubGraph { name: dir.name, url });
            if let Some(directive) = directives.next() {
                return Err((
                    format!(
                        "@join__graph directive used multiple times on subgraph {}",
                        value.value(),
                    ),
                    directive.name_span(),
                )
                    .into());
            }
        } else {
            sdl.subgraphs
                .insert(GraphName(value.value()), SdlSubGraph { name: None, url: None });
        }
    }
    Ok(())
}

fn ingest_extension_link_enum<'a>(sdl: &mut Sdl<'a>, ty: TypeDefinition<'a>) -> Result<(), Error> {
    let TypeDefinition::Enum(enm) = ty else {
        return Err(("extension__Link must be an enum type", ty.span()).into());
    };
    if !sdl.extensions.is_empty() {
        return Err(("extension__Link must be defined only once", enm.span()).into());
    }
    for value in enm.values() {
        let mut directives = value.directives().filter(|dir| dir.name() == "extension__link");
        let Some(directive) = directives.next() else {
            return Err((
                format!("Missing extension__link directive on extension {}", value.value()),
                value.span(),
            )
                .into());
        };
        let dir = directives::parse_extension_link(directive)?;
        let url = url::Url::parse(dir.url).map_err(|err| {
            (
                format!("Invalid url on subgraph {}: {err}", value.value()),
                directive.arguments_span(),
            )
        })?;
        sdl.extensions.insert(
            ExtensionName(value.value()),
            SdlExtension {
                url,
                directives: dir.schema_directives,
            },
        );
        if let Some(directive) = directives.next() {
            return Err((
                format!(
                    "@extension__link directive used multiple times on extension {}",
                    value.value(),
                ),
                directive.name_span(),
            )
                .into());
        }
    }
    Ok(())
}

fn finalize(sdl: &mut Sdl<'_>) -> Result<(), Error> {
    for (name, ext) in sdl.extensions.iter() {
        for (directive, span) in &ext.directives {
            if !sdl.subgraphs.contains_key(&directive.graph) {
                return Err((
                    format!(
                        "Unknown subgraph {} in @extension__link directive for extension {}",
                        directive.graph, name
                    ),
                    *span,
                )
                    .into());
            }
        }
    }

    Ok(())
}
