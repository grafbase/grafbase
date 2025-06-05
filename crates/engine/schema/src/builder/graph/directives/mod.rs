mod common;
mod composite;
mod federation;
mod resolvers;

use crate::builder::{Error, extension::ingest_extension_schema_directives, sdl};

use super::*;

pub(crate) struct DirectivesIngester<'a, 'sdl> {
    pub builder: &'a mut GraphBuilder<'sdl>,
    pub possible_composite_entity_keys:
        FxHashMap<(EntityDefinitionId, SubgraphId), Vec<PossibleCompositeEntityKey<'sdl>>>,
    pub for_operation_analytics_only: bool,
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
) -> Result<(), Error> {
    if !for_operation_analytics_only {
        ingest_extension_schema_directives(builder)?;
    }

    let mut ingester = DirectivesIngester {
        builder,
        possible_composite_entity_keys: Default::default(),
        for_operation_analytics_only,
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
        ingester.ingest_non_federation_aware_directives(def, &directives)?;

        // Interpret Federation directives
        ingester.ingest_federation_directives(def, &directives)?;
    }

    common::finalize_inaccessible(&mut ingester.graph);

    // Apollo federation entities, Composite Schema @lookup, extension, etc.
    if !for_operation_analytics_only {
        resolvers::generate(&mut ingester)?;
    }

    // Resolvers may change federation data, so we do this last.
    federation::add_not_fully_implemented_in(&mut ingester.graph);

    if !for_operation_analytics_only {
        composite::ingest_composite_field_directives_after_federation(&mut ingester)?;
    }

    Ok(())
}
