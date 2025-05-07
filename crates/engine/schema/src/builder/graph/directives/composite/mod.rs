mod is;
mod lookup;

use crate::{SubgraphId, builder::sdl};

use super::{DirectivesIngester, Error};

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

    pub(crate) fn ingest_composite_directive_after_federation(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        dir: sdl::Directive<'sdl>,
    ) -> Result<(), Error> {
        ingest_after_federation_directives(self, def, dir).map_err(|err| {
            err.with_prefix(format!(
                "At site {}, for directive @{}: ",
                def.to_site_string(self),
                dir.name()
            ))
        })
    }

    pub(crate) fn ingest_composite_lookup(
        &mut self,
        def: sdl::FieldSdlDefinition<'sdl>,
        subgraph_id: SubgraphId,
    ) -> Result<(), Error> {
        lookup::ingest(self, def, subgraph_id)
            .map_err(|err| err.with_prefix(format!("At site {}, for directive @lookup ", def.to_site_string(self))))
    }
}

fn ingest_before_federation_directives<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::SdlDefinition<'sdl>,
    dir: sdl::Directive<'sdl>,
) -> Result<(), Error> {
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

            // Nothing is ingested during this step, it's done when adding resolvers.
        }
        "composite__is" => {
            if !matches!(
                def,
                sdl::SdlDefinition::FieldDefinition(_) | sdl::SdlDefinition::ArgumentDefinition(_)
            ) {
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
        }
        "composite__require" => {
            if !matches!(def, sdl::SdlDefinition::ArgumentDefinition(_),) {
                return Err((
                    format!(
                        "invalid location: {}, expected one of: {}",
                        def.location().as_str(),
                        [sdl::DirectiveLocation::ArgumentDefinition.as_str()].join(", ")
                    ),
                    def.span(),
                )
                    .into());
            }
        }
        _ => return Err("unknown or unsupported directive".into()),
    }
    Ok(())
}

fn ingest_after_federation_directives<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::SdlDefinition<'sdl>,
    dir: sdl::Directive<'sdl>,
) -> Result<(), Error> {
    match (dir.name(), def) {
        ("composite__is", sdl::SdlDefinition::FieldDefinition(def)) => is::ingest_field(ingester, def, dir),
        _ => Ok(()),
    }
}
