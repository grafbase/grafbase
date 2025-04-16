mod common;
mod composite;
mod federation;
mod resolvers;

use crate::builder::{Error, sdl};

use super::*;

pub(crate) struct DirectivesIngester<'a, 'sdl> {
    pub builder: &'a mut GraphBuilder<'sdl>,
    pub sdl_definitions: &'a sdl::SdlDefinitions<'sdl>,
    pub composite_entity_keys: FxHashMap<(EntityDefinitionId, SubgraphId), Vec<FieldSetRecord>>,
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
    sdl_definitions: &sdl::SdlDefinitions<'a>,
) -> Result<(), Error> {
    let mut ingester = DirectivesIngester {
        builder,
        sdl_definitions,
        composite_entity_keys: Default::default(),
    };

    let mut directives = Vec::new();
    for def in ingester.sdl_definitions.values().copied() {
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
    resolvers::generate(&mut ingester)?;

    // Resolvers may change federation data, so we do this last.
    federation::add_not_fully_implemented_in(&mut ingester.graph);

    Ok(())
}
