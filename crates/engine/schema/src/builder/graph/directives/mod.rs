mod cache;
mod common;
mod composite;
mod federation;
mod resolvers;

use crate::builder::{
    Error, extension::ingest_extension_schema_directives, graph::directives::cache::CachedJoinTypeDirective, sdl,
};

use super::*;
pub(in crate::builder) use common::finalize_inaccessible;
use cynic_parser::Span;
use cynic_parser_deser::ConstDeserializer as _;

pub(crate) type PossibleCompositeEntityKeys<'sdl> =
    FxHashMap<(EntityDefinitionId, SubgraphId), Vec<PossibleCompositeEntityKey<'sdl>>>;

pub(crate) struct DirectivesIngester<'a, 'sdl> {
    pub builder: &'a mut GraphBuilder<'sdl>,
    pub possible_composite_entity_keys: PossibleCompositeEntityKeys<'sdl>,
    pub for_operation_analytics_only: bool,
    pub errors: Vec<Error>,
    pub cache: FxHashMap<HashableSpan, Option<CachedJoinTypeDirective<'sdl>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashableSpan {
    start: usize,
    end: usize,
}

impl From<Span> for HashableSpan {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub(crate) fn get_join_type(&mut self, dir: sdl::Directive<'sdl>) -> Option<CachedJoinTypeDirective<'sdl>> {
        if dir.name() != "join__type" {
            return None;
        }

        self.cache
            .entry(dir.name_span().into())
            .or_insert_with(|| match dir.deserialize::<sdl::JoinTypeDirective<'sdl>>() {
                Ok(join_type) => match self.builder.subgraphs.try_get(join_type.graph, dir.arguments_span()) {
                    Ok(subgraph_id) => Some(CachedJoinTypeDirective {
                        subgraph_id,
                        key: join_type.key,
                        resolvable: join_type.resolvable,
                        is_interface_object: join_type.is_interface_object,
                        arguments_span: dir.arguments_span(),
                    }),
                    Err(err) => {
                        self.errors.push(err);
                        None
                    }
                },
                Err(err) => {
                    self.errors.push(Error::new(err).span(dir.arguments_span()));
                    None
                }
            })
            .clone()
    }
}

pub(crate) struct PossibleCompositeEntityKey<'sdl> {
    key: FieldSetRecord,
    key_str: &'sdl str,
    used_by: Option<sdl::FieldSdlDefinition<'sdl>>,
}

impl<'sdl> std::ops::Deref for DirectivesIngester<'_, 'sdl> {
    type Target = GraphBuilder<'sdl>;
    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl std::ops::DerefMut for DirectivesIngester<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.builder
    }
}

pub(crate) fn ingest_directives<'a>(
    builder: &mut GraphBuilder<'a>,
    for_operation_analytics_only: bool,
) -> Result<(), Vec<Error>> {
    if !for_operation_analytics_only {
        ingest_extension_schema_directives(builder)?;
    }

    let mut ingester = DirectivesIngester {
        builder,
        possible_composite_entity_keys: Default::default(),
        for_operation_analytics_only,
        errors: Vec::new(),
        cache: FxHashMap::default(),
    };

    let mut directives = Vec::new();
    for def in ingester.definitions.clone().site_id_to_sdl.values().copied() {
        directives.clear();
        directives.extend(def.directives());
        if let Some(ext) = def
            .as_type()
            .and_then(|ty| ingester.builder.sdl.type_extensions.get(ty.name()))
        {
            directives.extend(ext.iter().flat_map(|ext| ext.directives()));
        }

        // Add non-federation-aware directives, including extensions
        ingester.ingest_non_federation_aware_directives(def, &directives);

        // Interpret Federation directives
        ingester.ingest_federation_directives(def, &directives);
    }

    if !ingester.errors.is_empty() {
        return Err(ingester.errors);
    }

    common::finalize_inaccessible(&mut ingester.graph);

    // Apollo federation entities, Composite Schema @lookup, extension, etc.
    if !for_operation_analytics_only {
        resolvers::generate(&mut ingester);
        if !ingester.errors.is_empty() {
            return Err(ingester.errors);
        }
    }

    // Resolvers may change federation data, so we do this last.
    federation::add_not_fully_implemented_in(&mut ingester.graph);

    if !for_operation_analytics_only {
        composite::ingest_composite_field_directives_after_federation_and_resolvers(&mut ingester)
    }

    if !ingester.errors.is_empty() {
        Err(ingester.errors)
    } else {
        Ok(())
    }
}
