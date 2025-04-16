use cynic_parser_deser::ConstDeserializer as _;

use crate::{
    ListSizeDirectiveRecord, TypeDefinitionId, TypeSystemDirectiveId,
    builder::{Error, graph::directives::DirectivesIngester, sdl},
};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn create_list_size_directive(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        directive: sdl::Directive<'sdl>,
    ) -> Result<TypeSystemDirectiveId, Error> {
        let sdl::SdlDefinition::FieldDefinition(def) = def else {
            return Err((
                format!("Invalid @listSize directive location: {}", def.location()),
                directive.name_span(),
            )
                .into());
        };
        let dir = directive.deserialize::<sdl::ListSizeDirective>().map_err(|err| {
            (
                format!("Invalid @listSize directive: {}", err),
                directive.arguments_span(),
            )
        })?;
        let slicing_argument_ids = {
            let field_argument_ids = self.graph[def.id].argument_ids;
            dir.slicing_arguments
                .into_iter()
                .map(|name| {
                    field_argument_ids
                        .into_iter()
                        .find(|id| self.ctx[self.graph[*id].name_id] == name)
                        .ok_or_else(|| {
                            (
                                format!("Invalid @listSize directive slicing_argument: {}", name),
                                directive.arguments_span(),
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()
        }?;
        let sized_field_ids = if !dir.sized_fields.is_empty() {
            let output_field_ids = match self.graph[def.id].ty_record.definition_id {
                TypeDefinitionId::Interface(id) => self.graph[id].field_ids,
                TypeDefinitionId::Object(id) => self.graph[id].field_ids,
                _ => {
                    return Err((
                        "sized_fields can only be used with a interface/object output type",
                        directive.arguments_span(),
                    )
                        .into());
                }
            };
            dir.sized_fields
                .into_iter()
                .map(|name| {
                    output_field_ids
                        .into_iter()
                        .find(|id| self.ctx[self.graph[*id].name_id] == name)
                        .ok_or_else(|| {
                            (
                                format!("Invalid @listSize directive sized_field: {}", name),
                                directive.arguments_span(),
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        self.graph.list_size_directives.push(ListSizeDirectiveRecord {
            assumed_size: dir.assumed_size,
            slicing_argument_ids,
            sized_field_ids,
            require_one_slicing_argument: dir.require_one_slicing_argument,
        });
        Ok(TypeSystemDirectiveId::ListSize(
            (self.graph.list_size_directives.len() - 1).into(),
        ))
    }
}
