use wrapping::Wrapping;

use crate::{
    EntityDefinitionId, FieldDefinitionId, Graph, InputObjectDefinitionId, InputValueDefinitionId, SchemaInputValueId,
    StringId, SubgraphId, TypeDefinitionId, builder::GraphBuilder,
};

pub(super) trait Target: Copy {
    type Id: Copy + Eq;
    fn id(self) -> Self::Id;
    fn display(self, ctx: &GraphBuilder<'_>) -> String;
    fn type_definition(self, graph: &Graph) -> TypeDefinitionId;
    fn fields(self, ctx: &GraphBuilder<'_>) -> Vec<(StringId, (Self, Wrapping))>;
    fn on_missing_field(self, ctx: &GraphBuilder<'_>) -> OnMissingField;
}

pub(super) enum OnMissingField {
    None,
    DefaultValue(SchemaInputValueId),
    Providable,
}

#[derive(Clone, Copy)]
pub(super) enum InputTarget {
    InputField {
        input_object_id: InputObjectDefinitionId,
        input_field_id: InputValueDefinitionId,
    },
    Argument {
        field_id: FieldDefinitionId,
        argument_id: InputValueDefinitionId,
    },
}

impl Target for InputTarget {
    type Id = InputValueDefinitionId;

    fn id(self) -> Self::Id {
        match self {
            InputTarget::InputField { input_field_id, .. } => input_field_id,
            InputTarget::Argument { argument_id, .. } => argument_id,
        }
    }

    fn display(self, ctx: &GraphBuilder<'_>) -> String {
        match self {
            InputTarget::InputField {
                input_object_id,
                input_field_id,
            } => {
                let input_object = &ctx.graph[input_object_id];
                let field = &ctx.graph[input_field_id];
                format!("{}.{}", &ctx[input_object.name_id], &ctx[field.name_id])
            }
            InputTarget::Argument { field_id, argument_id } => {
                let field = &ctx.graph[field_id];
                let argument = &ctx.graph[argument_id];
                format!(
                    "{}.{}.{}",
                    ctx[ctx.definition_name_id(field.parent_entity_id.into())],
                    ctx[field.name_id],
                    ctx[argument.name_id]
                )
            }
        }
    }

    fn type_definition(self, graph: &Graph) -> TypeDefinitionId {
        graph[self.id()].ty_record.definition_id
    }

    fn fields(self, ctx: &GraphBuilder<'_>) -> Vec<(StringId, (Self, Wrapping))> {
        ctx.graph[self.id()]
            .ty_record
            .definition_id
            .as_input_object()
            .map(|input_object_id| {
                ctx.graph[input_object_id]
                    .input_field_ids
                    .into_iter()
                    .map(|id| {
                        (
                            ctx.graph[id].name_id,
                            (
                                InputTarget::InputField {
                                    input_object_id,
                                    input_field_id: id,
                                },
                                ctx.graph[id].ty_record.wrapping,
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn on_missing_field(self, ctx: &GraphBuilder<'_>) -> OnMissingField {
        match ctx.graph[self.id()].default_value_id {
            Some(default_value_id) => OnMissingField::DefaultValue(default_value_id),
            None => OnMissingField::None,
        }
    }
}

impl Target for (SubgraphId, FieldDefinitionId) {
    type Id = FieldDefinitionId;

    fn id(self) -> Self::Id {
        self.1
    }

    fn display(self, ctx: &GraphBuilder<'_>) -> String {
        let field = &ctx.graph[self.id()];
        format!(
            "{}.{}",
            ctx[ctx.definition_name_id(field.parent_entity_id.into())],
            ctx[field.name_id],
        )
    }

    fn type_definition(self, graph: &Graph) -> TypeDefinitionId {
        graph[self.id()].ty_record.definition_id
    }

    fn fields(self, ctx: &GraphBuilder<'_>) -> Vec<(StringId, (Self, Wrapping))> {
        let (subgraph_id, field_id) = self;
        ctx.graph[field_id]
            .ty_record
            .definition_id
            .as_entity()
            .map(|entity_id| {
                let field_ids = match entity_id {
                    EntityDefinitionId::Interface(id) => ctx.graph[id].field_ids,
                    EntityDefinitionId::Object(id) => ctx.graph[id].field_ids,
                };
                field_ids
                    .into_iter()
                    .filter(|id| ctx.graph[*id].exists_in_subgraph_ids.contains(&subgraph_id))
                    .map(|id| {
                        (
                            ctx.graph[id].name_id,
                            ((subgraph_id, id), ctx.graph[id].ty_record.wrapping),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn on_missing_field(self, ctx: &GraphBuilder<'_>) -> OnMissingField {
        let (subgraph_id, field_id) = self;
        if ctx.graph[field_id]
            .resolver_ids
            .iter()
            .any(|id| ctx.get_subgraph_id(*id) == subgraph_id)
        {
            OnMissingField::Providable
        } else {
            OnMissingField::None
        }
    }
}
