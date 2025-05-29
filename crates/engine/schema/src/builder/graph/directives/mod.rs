mod common;
mod composite;
mod federation;
mod resolvers;
mod supergraph;

use cynic_parser_deser::ConstDeserializer as _;
use itertools::Itertools as _;

use crate::builder::{BoundSelectedValue, Error, extension::ingest_extension_schema_directives, sdl};

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
        for def in ingester.definitions.clone().site_id_to_sdl.values().copied() {
            directives.clear();
            directives.extend(def.directives());
            if let Some(ext) = def
                .as_type()
                .and_then(|ty| ingester.builder.sdl.type_extensions.get(ty.name()))
            {
                directives.extend(ext.iter().flat_map(|ext| ext.directives()));
            }

            ingester.ingest_federation_aware_directives(def, &directives)?;
        }
    }

    Ok(())
}

impl GraphBuilder<'_> {
    pub fn find_field_selection_map<'d>(
        &mut self,
        subgraph_name: sdl::GraphName<'_>,
        source: TypeRecord,
        field_definition_id: FieldDefinitionId,
        argument_id: InputValueDefinitionId,
        directives: impl Iterator<Item = sdl::Directive<'d>>,
    ) -> Result<Option<(BoundSelectedValue<InputValueDefinitionId>, sdl::Directive<'d>)>, Error> {
        let mut is_directives = directives
            .filter(|dir| dir.name() == "composite__is")
            .map(|dir| {
                dir.deserialize::<sdl::IsDirective>()
                    .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                    .map(|args| (dir, args))
            })
            .filter_ok(|(_, args)| args.graph == subgraph_name);

        let Some((field_selection_map, is_directive)) = is_directives
            .next()
            .transpose()?
            .map(
                |(
                    is_directive,
                    sdl::IsDirective {
                        field: field_selection_map,
                        ..
                    },
                )| {
                    tracing::trace!(
                        "Found @is(field: \"{field_selection_map}\") for {}",
                        self.ctx[self.graph[argument_id].name_id]
                    );
                    self.parse_field_selection_map_for_argument(
                        source,
                        field_definition_id,
                        argument_id,
                        field_selection_map,
                    )
                    .map(|field_selection_map| (field_selection_map, is_directive))
                    .map_err(|err| {
                        (
                            format!("for associated @is directive: {err}"),
                            is_directive.arguments_span(),
                        )
                    })
                },
            )
            .transpose()?
        else {
            return Ok(None);
        };

        if is_directives.next().is_some() {
            return Err((
                "Multiple @composite__is directives on the same argument are not supported.",
                is_directive.name_span(),
            )
                .into());
        }

        Ok(Some((field_selection_map, is_directive)))
    }
}
