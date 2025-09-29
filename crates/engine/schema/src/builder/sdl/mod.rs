mod definitions;
mod directives;
mod link_directive;
mod span;
mod wrapping;

pub(crate) use cynic_parser::{
    ConstValue,
    common::{TypeWrappersIter, WrappingType},
    type_system::*,
};
use cynic_parser_deser::ConstDeserializer as _;
use rapidhash::fast::RapidHashMap;

pub(crate) use self::wrapping::*;
pub(crate) use definitions::*;
pub(crate) use directives::*;
pub(crate) use link_directive::*;
pub(crate) use span::*;

use super::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash, id_derives::Id)]
pub(crate) struct LinkId(u16);

#[derive(Default, id_derives::IndexedFields)]
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
    pub subgraphs: RapidHashMap<GraphName<'a>, SdlSubGraph<'a>>,
    pub extensions: RapidHashMap<ExtensionName<'a>, SdlExtension<'a>>,
    pub directive_namespaces: RapidHashMap<String, LinkId>,
    pub directive_imports: RapidHashMap<&'a str, DirectiveImport<'a>>,
    #[indexed_by(LinkId)]
    pub links: Vec<LinkDirective<'a>>,
    pub schema_directives: Vec<Directive<'a>>,
}

impl<'a> Sdl<'a> {
    pub fn iter_links(&self) -> impl Iterator<Item = (LinkId, &LinkDirective<'a>)> {
        self.links.iter().enumerate().map(|(i, link)| (LinkId::from(i), link))
    }
}

impl std::ops::Index<cynic_parser::Span> for Sdl<'_> {
    type Output = str;

    fn index(&self, span: cynic_parser::Span) -> &Self::Output {
        &self.raw[span.start..span.end]
    }
}

#[derive(Debug)]
pub(crate) struct DirectiveImport<'a> {
    pub link_id: LinkId,
    pub original_name: &'a str,
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
    type Error = Vec<Error>;

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
            subgraphs: RapidHashMap::default(),
            extensions: RapidHashMap::default(),
            directive_namespaces: Default::default(),
            directive_imports: Default::default(),
            links: Default::default(),
            schema_directives: Default::default(),
        };
        let mut errors = Vec::new();
        let mut schema_definition = None;
        let mut all_schema_definitions = Vec::new();

        for def in doc.definitions() {
            match def {
                Definition::Schema(def) => {
                    if schema_definition.is_some() {
                        errors
                            .push(Error::new("A document must include at most one schema definition").span(def.span()));
                    } else {
                        schema_definition = Some(def);
                        all_schema_definitions.push(def);
                        let last_ix = all_schema_definitions.len() - 1;
                        all_schema_definitions.swap(0, last_ix);
                    }
                }
                Definition::SchemaExtension(def) => {
                    all_schema_definitions.push(def);
                }
                Definition::Type(type_definition) => {
                    if let Some(name) = type_definition.name().strip_prefix("join__") {
                        match name {
                            "Graph" => {
                                if let Err(err) = ingest_join_graph_enum(&mut sdl, type_definition) {
                                    errors.push(err);
                                }
                            }
                            "FieldSet" => {}
                            _ => {
                                errors.push(
                                    Error::new(format!("join__{name} is an unknown federation type."))
                                        .span(type_definition.span()),
                                );
                            }
                        }
                    } else if let Some(name) = type_definition.name().strip_prefix("extension__") {
                        match name {
                            "Link" => {
                                if let Err(err) = ingest_extension_link_enum(&mut sdl, type_definition) {
                                    errors.push(err);
                                }
                            }
                            _ => {
                                errors.push(
                                    Error::new(format!("extension__{name} is an unknown extension type."))
                                        .span(type_definition.span()),
                                );
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
                    let name = directive_definition.name();
                    if !name.starts_with("join__") && name != "core" && !name.starts_with("composite__") {
                        tracing::warn!("Directive definitions are ignored.")
                    }
                }
            }
        }

        for schema in all_schema_definitions {
            for directive in schema.directives() {
                if directive.name() != "link" {
                    sdl.schema_directives.push(directive);
                    continue;
                }

                let link = match directive.deserialize::<LinkDirective<'a>>() {
                    Ok(link) => link,
                    Err(err) => {
                        errors.push(
                            Error::new(format!("Could not parse @link directive: {err}")).span(directive.name_span()),
                        );
                        continue;
                    }
                };

                let link_id = LinkId::from(sdl.links.len());

                if let Some(namespace) = link.namespace.clone() {
                    sdl.directive_namespaces.insert(namespace, link_id);
                }

                for import in link.import.as_ref().map(|import| import.iter()).unwrap_or_default() {
                    match import {
                        Import::String(name) | Import::Qualified(QualifiedImport { name, r#as: None }) => {
                            let name = name.strip_prefix('@').unwrap_or(name);
                            sdl.directive_imports.insert(
                                name,
                                DirectiveImport {
                                    link_id,
                                    original_name: name,
                                },
                            )
                        }
                        Import::Qualified(QualifiedImport {
                            name: original_name,
                            r#as: Some(name),
                        }) => {
                            let name = name.strip_prefix('@').unwrap_or(name);
                            let original_name = original_name.strip_prefix('@').unwrap_or(original_name);
                            sdl.directive_imports
                                .insert(name, DirectiveImport { link_id, original_name })
                        }
                    };
                }

                sdl.links.push(link);
            }
            for root_type in schema.root_operations() {
                match root_type.operation_type() {
                    cynic_parser::common::OperationType::Query => {
                        if sdl.root_types.query.is_some() {
                            errors.push(
                                Error::new("A document must include at most one query root type")
                                    .span(root_type.span()),
                            );
                        } else {
                            sdl.root_types.query = Some(root_type.named_type());
                        }
                    }
                    cynic_parser::common::OperationType::Mutation => {
                        if sdl.root_types.mutation.is_some() {
                            errors.push(
                                Error::new("A document must include at most one mutation root type")
                                    .span(root_type.span()),
                            );
                        } else {
                            sdl.root_types.mutation = Some(root_type.named_type());
                        }
                    }
                    cynic_parser::common::OperationType::Subscription => {
                        if sdl.root_types.subscription.is_some() {
                            errors.push(
                                Error::new("A document must include at most one subscription root type")
                                    .span(root_type.span()),
                            );
                        } else {
                            sdl.root_types.subscription = Some(root_type.named_type());
                        }
                    }
                }
            }
        }

        if let Err(err) = finalize(&mut sdl) {
            errors.push(err);
        }

        if !errors.is_empty() { Err(errors) } else { Ok(sdl) }
    }
}

fn ingest_join_graph_enum<'a>(sdl: &mut Sdl<'a>, ty: TypeDefinition<'a>) -> Result<(), Error> {
    let TypeDefinition::Enum(enm) = ty else {
        return Err(Error::new("join__Graph must be an enum type").span(ty.span()));
    };
    if !sdl.subgraphs.is_empty() {
        return Err(Error::new("join__Graph must be defined only once").span(enm.span()));
    }
    let mut errors = Vec::new();
    for value in enm.values() {
        let mut directives = value.directives().filter(|dir| dir.name() == "join__graph");
        if let Some(directive) = directives.next() {
            match directive.deserialize::<JoinGraphDirective<'_>>() {
                Ok(dir) => {
                    let url = match dir.url {
                        Some(url_str) => match url::Url::parse(url_str) {
                            Ok(url) => Some(url),
                            Err(err) => {
                                errors.push(
                                    Error::new(format!("Invalid url on subgraph {}: {err}", value.value()))
                                        .span(directive.arguments_span()),
                                );
                                None
                            }
                        },
                        None => None,
                    };
                    if errors.is_empty() {
                        sdl.subgraphs
                            .insert(GraphName(value.value()), SdlSubGraph { name: dir.name, url });
                    }
                    if let Some(directive) = directives.next() {
                        errors.push(
                            Error::new(format!(
                                "@join__graph directive used multiple times on subgraph {}",
                                value.value(),
                            ))
                            .span(directive.name_span()),
                        );
                    }
                }
                Err(err) => {
                    errors.push(
                        Error::new(format!(
                            "Invalid @join__graph directive on subgraph {}: {err}",
                            value.value()
                        ))
                        .span(directive.arguments_span()),
                    );
                }
            }
        } else {
            sdl.subgraphs
                .insert(GraphName(value.value()), SdlSubGraph { name: None, url: None });
        }
    }
    if !errors.is_empty() {
        return Err(errors.into_iter().next().unwrap());
    }
    Ok(())
}

fn ingest_extension_link_enum<'a>(sdl: &mut Sdl<'a>, ty: TypeDefinition<'a>) -> Result<(), Error> {
    let TypeDefinition::Enum(enm) = ty else {
        return Err(Error::new("extension__Link must be an enum type").span(ty.span()));
    };
    if !sdl.extensions.is_empty() {
        return Err(Error::new("extension__Link must be defined only once").span(enm.span()));
    }
    let mut errors = Vec::new();
    for value in enm.values() {
        let mut directives = value.directives().filter(|dir| dir.name() == "extension__link");
        let Some(directive) = directives.next() else {
            errors.push(
                Error::new(format!(
                    "Missing extension__link directive on extension {}",
                    value.value()
                ))
                .span(value.span()),
            );
            continue;
        };
        match directives::parse_extension_link(directive) {
            Ok(dir) => match url::Url::parse(dir.url) {
                Ok(url) => {
                    sdl.extensions.insert(
                        ExtensionName(value.value()),
                        SdlExtension {
                            url,
                            directives: dir.schema_directives,
                        },
                    );
                }
                Err(err) => {
                    errors.push(
                        Error::new(format!("Invalid url on extension {}: {err}", value.value()))
                            .span(directive.arguments_span()),
                    );
                }
            },
            Err(err) => {
                errors.push(err);
            }
        }
        if let Some(directive) = directives.next() {
            errors.push(
                Error::new(format!(
                    "@extension__link directive used multiple times on extension {}",
                    value.value(),
                ))
                .span(directive.name_span()),
            );
        }
    }
    if !errors.is_empty() {
        return Err(errors.into_iter().next().unwrap());
    }
    Ok(())
}

fn finalize(sdl: &mut Sdl<'_>) -> Result<(), Error> {
    for (name, ext) in sdl.extensions.iter() {
        for (directive, span) in &ext.directives {
            if !sdl.subgraphs.contains_key(&directive.graph) {
                return Err(Error::new(format!(
                    "Unknown subgraph {} in @extension__link directive for extension {}",
                    directive.graph, name
                ))
                .span(*span));
            }
        }
    }

    Ok(())
}
