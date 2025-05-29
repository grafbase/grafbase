mod derive;
mod injection;
mod lookup;
mod require;

use crate::{DirectiveSiteId, builder::sdl};

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
        ("composite__derive", sdl::SdlDefinition::FieldDefinition(def)) => derive::ingest(ingester, def, dir),
        _ => Ok(()),
    }
}
