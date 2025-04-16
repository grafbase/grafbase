mod key;
mod lookup;

use serde::Deserialize;

use crate::{
    SubgraphId,
    builder::sdl::{self, ConstValue, ConstValueArgumentsDeserializer},
};

use super::{DirectivesIngester, Error};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub(crate) fn ingest_composite_schema_directive(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        subgraph_id: SubgraphId,
        name: &str,
        arguments: Option<ConstValue<'sdl>>,
    ) -> Result<(), Error> {
        dispatch(self, def, subgraph_id, name, arguments)
            .map_err(|err| err.with_prefix(format!("At site {}, for directive @{name} ", def.to_site_string(self))))
    }

    pub(crate) fn ingest_composite_schema_lookup(
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
    subgraph_id: SubgraphId,
    name: &str,
    arguments: Option<ConstValue<'sdl>>,
) -> Result<(), Error> {
    match name {
        "key" => {
            let Some(entity) = def.as_entity() else {
                return Err((
                    format!(
                        "invalid location: {}, expected one of: {}",
                        def.location().as_str(),
                        [
                            sdl::DirectiveLocation::Object.as_str(),
                            sdl::DirectiveLocation::Interface.as_str(),
                        ]
                        .join(", ")
                    ),
                    def.span(),
                )
                    .into());
            };

            key::ingest(ingester, entity, subgraph_id, deserialize(arguments)?)?;
        }
        "lookup" => {
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

fn deserialize<'de, T: Deserialize<'de>>(arguments: Option<ConstValue<'de>>) -> Result<T, String> {
    serde_path_to_error::deserialize(ConstValueArgumentsDeserializer(arguments)).map_err(|err| {
        let path = err.path().to_string();
        let err = err.into_inner().to_string();
        format!("Invalid directive arguments, at {}: {}", path, err)
    })
}
