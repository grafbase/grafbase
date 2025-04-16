use serde::Deserialize;

use crate::{
    SubgraphId,
    builder::{
        GraphBuilder,
        sdl::{self, ConstValue, ConstValueArgumentsDeserializer},
    },
};

impl<'a> GraphBuilder<'a> {
    pub(crate) fn ingest_composite_schema_directive(
        &mut self,
        def: sdl::SdlDefinition<'a>,
        subgraph_id: SubgraphId,
        name: &str,
        arguments: Option<ConstValue<'a>>,
    ) -> Result<(), String> {
        match name {
            "key" => {
                let Some(entity) = def.as_entity() else {
                    return Err(format!(
                        "Invalid directive location: {}, expected one of: {}",
                        def.location().as_str(),
                        [
                            sdl::DirectiveLocation::Object.as_str(),
                            sdl::DirectiveLocation::Interface.as_str(),
                        ]
                        .join(", ")
                    ));
                };

                let KeyDirectiveArguments { fields } = deserialize(arguments)?;
                let fields = self
                    .parse_field_set(entity.id().into(), fields)
                    .map_err(|err| format!("Invalid SelectionSet for argument 'fields': {}", err))?;
                self.composite_entity_keys
                    .entry((entity.id(), subgraph_id))
                    .or_default()
                    .push(fields);
            }
            _ => {
                return Err("Unknown directive".to_string());
            }
        }

        Ok(())
    }
}

fn deserialize<'de, T: Deserialize<'de>>(arguments: Option<ConstValue<'de>>) -> Result<T, String> {
    serde_path_to_error::deserialize(ConstValueArgumentsDeserializer(arguments)).map_err(|err| {
        let path = err.path().to_string();
        let err = err.into_inner().to_string();
        format!("Invalid directive arguments, at {}: {}", path, err)
    })
}

#[derive(serde::Deserialize)]
struct KeyDirectiveArguments<'a> {
    #[serde(borrow)]
    fields: &'a str,
}
