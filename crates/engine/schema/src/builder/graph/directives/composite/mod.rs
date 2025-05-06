mod lookup;

use cynic_parser_deser::ConstDeserializer as _;

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
        "composite__is" => match def {
            sdl::SdlDefinition::FieldDefinition(def) => {
                let sdl::IsDirective {
                    graph,
                    field_selection_map,
                } = dir.deserialize().map_err(|err| {
                    (
                        format!(
                            "At {}, invalid composite__lookup directive: {}",
                            def.to_site_string(ingester),
                            err
                        ),
                        dir.arguments_span(),
                    )
                })?;
                let _subgraph_id = ingester.subgraphs.try_get(graph, dir.arguments_span())?;
                let output = ingester.graph[def.id].parent_entity_id;
                ingester.parse_field_selection_map_for_field(output, def.id, field_selection_map)?;
            }
            sdl::SdlDefinition::ArgumentDefinition(def) => {
                let sdl::IsDirective {
                    graph,
                    field_selection_map,
                } = dir.deserialize().map_err(|err| {
                    (
                        format!(
                            "At {}, invalid composite__lookup directive: {}",
                            def.to_site_string(ingester),
                            err
                        ),
                        dir.arguments_span(),
                    )
                })?;
                let _subgraph_id = ingester.subgraphs.try_get(graph, dir.arguments_span())?;
                let output = ingester.graph[def.field_id]
                    .ty_record
                    .definition_id
                    .as_entity()
                    .unwrap();
                ingester.parse_field_selection_map_for_argument(output, def.field_id, def.id, field_selection_map)?;
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
