mod derive;
mod injection;
mod lookup;
mod require;

use cynic_parser_deser::ConstDeserializer as _;

use crate::{DirectiveSiteId, builder::sdl};

use super::{DirectivesIngester, Error, FieldDefinitionId};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub(crate) fn ingest_composite_directive_before_federation(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        dir: sdl::Directive<'sdl>,
    ) -> Result<(), Error> {
        ingest_before_federation_directives(self, def, dir).map_err(|err| {
            err.with_prefix(format!(
                "At site {}, for directive @{}:",
                def.to_site_string(self),
                dir.name(),
            ))
        })
    }

    pub(crate) fn ingest_composite_lookup(
        &mut self,
        def: sdl::FieldSdlDefinition<'sdl>,
        directive: sdl::Directive<'sdl>,
    ) -> Result<(), Error> {
        lookup::ingest(self, def, directive)
            .map_err(|err| err.with_prefix(format!("At site {}, for directive @lookup ", def.to_site_string(self))))
    }
}

fn ingest_before_federation_directives<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::SdlDefinition<'sdl>,
    dir: sdl::Directive<'sdl>,
) -> Result<(), Error> {
    // Nothing is ingested during this step, it's done when adding resolvers.
    match dir.name() {
        "composite__lookup" => {
            let Some(field) = def.as_field() else {
                return Err((
                    format!(
                        "invalid location: {}, expected one of: {}",
                        def.location().as_str(),
                        [sdl::DirectiveLocation::Field.as_str(),].join(", ")
                    ),
                    def.span(),
                )
                    .into());
            };

            if ingester.graph[field.id].parent_entity_id != ingester.graph.root_operation_types_record.query_id.into() {
                return Err((
                    "can only be used on fields of the root query type".to_string(),
                    field.span(),
                )
                    .into());
            }
        }
        "composite__derive" => {
            if !matches!(def, sdl::SdlDefinition::FieldDefinition(_)) {
                return Err((
                    format!(
                        "invalid location: {}, expected one of: {}",
                        def.location().as_str(),
                        [sdl::DirectiveLocation::Field.as_str(),].join(", ")
                    ),
                    def.span(),
                )
                    .into());
            };
        }
        "composite__is" => match def {
            sdl::SdlDefinition::FieldDefinition(def) => {
                if ingester.definitions.site_id_to_sdl[&DirectiveSiteId::Field(def.id)]
                    .directives()
                    .all(|dir| dir.name() != "composite__derive")
                {
                    return Err((
                        "@is can only be used on a field in conjonction with @derive directive",
                        def.span(),
                    )
                        .into());
                }
            }
            sdl::SdlDefinition::ArgumentDefinition(def) => {
                if ingester.definitions.site_id_to_sdl[&DirectiveSiteId::Field(def.field_id)]
                    .directives()
                    .all(|dir| dir.name() != "composite__lookup")
                {
                    return Err((
                        "@is can only be used on a argument in conjonction with @lookup directive on the field",
                        def.span(),
                    )
                        .into());
                }
            }
            _ => {
                return Err((
                    format!(
                        "invalid location: {}, expected one of: {}",
                        def.location().as_str(),
                        [
                            sdl::DirectiveLocation::Field.as_str(),
                            sdl::DirectiveLocation::ArgumentDefinition.as_str()
                        ]
                        .join(", ")
                    ),
                    def.span(),
                )
                    .into());
            }
        },
        "composite__require" => {
            let sdl::SdlDefinition::ArgumentDefinition(def) = def else {
                return Err((
                    format!(
                        "invalid location: {}, expected one of: {}",
                        def.location().as_str(),
                        [sdl::DirectiveLocation::ArgumentDefinition.as_str()].join(", ")
                    ),
                    def.span(),
                )
                    .into());
            };
            let sdl::RequireDirective { graph, .. } = dir.deserialize().map_err(|err| {
                (
                    format!(
                        "At {}, invalid composite__require directive: {}",
                        def.to_site_string(ingester),
                        err
                    ),
                    dir.arguments_span(),
                )
            })?;
            let subgraph_id = ingester.subgraphs.try_get(graph, dir.arguments_span())?;
            if ingester.graph[def.id].is_internal_in_id.is_some() {
                return Err((
                    "cannot use @require multiple times on a argument within a subgraph".to_string(),
                    dir.name_span(),
                )
                    .into());
            }
            ingester.graph[def.id].is_internal_in_id = Some(subgraph_id);
        }
        _ => return Err("unknown or unsupported directive".into()),
    }
    Ok(())
}

pub(crate) fn ingest_composite_field_directives_after_federation_and_resolvers(
    ingester: &mut DirectivesIngester<'_, '_>,
) -> Result<(), Error> {
    for id in 0..ingester.graph.field_definitions.len() {
        let id = FieldDefinitionId::from(id);
        let Some(&sdl::SdlDefinition::FieldDefinition(field)) = ingester.definitions.site_id_to_sdl.get(&id.into())
        else {
            // Introspection fields aren't part of the SDL.
            continue;
        };
        require::ingest_field(ingester, field).map_err(|err| {
            err.with_prefix(format!("At site {}: ", field.to_site_string(ingester)))
                .with_span_if_absent(field.name_span())
        })?;
        for directive in field.directives() {
            if let "composite__derive" = directive.name() {
                derive::ingest(ingester, field, directive).map_err(|err| {
                    err.with_prefix(format!(
                        "At site {}, for directive @composite__derive: ",
                        field.to_site_string(ingester)
                    ))
                    .with_span_if_absent(directive.arguments_span())
                })?
            }
        }
    }
    Ok(())
}
