mod lookup;

use crate::{SubgraphId, builder::sdl};

use super::{DirectivesIngester, Error};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub(crate) fn ingest_composite_directive(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        dir: sdl::Directive<'sdl>,
    ) -> Result<(), Error> {
        dispatch(self, def, dir).map_err(|err| {
            err.with_prefix(format!(
                "At site {}, for directive @{} ",
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

fn dispatch<'sdl>(
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
        _ => return Err("unknown or unsupported directive".into()),
    }

    Ok(())
}
