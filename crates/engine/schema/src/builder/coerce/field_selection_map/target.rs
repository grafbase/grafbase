use wrapping::Wrapping;

use crate::{
    EntityDefinitionId, FieldDefinitionId, Graph, InputValueDefinitionId, InputValueParentDefinitionId,
    SchemaInputValueId, StringId, SubgraphId, TypeDefinitionId, builder::GraphBuilder,
};

/// The wrapping is separated from the Target because we remove it when binding lists
pub(super) trait TargetField: Copy {
    type Id: Copy + Eq;
    fn id(self) -> Self::Id;
    fn id_display<'g>(id: Self::Id, ctx: &'g GraphBuilder<'_>) -> &'g str;
    fn display(self, ctx: &GraphBuilder<'_>) -> String;
    fn type_definition(self, graph: &Graph) -> TypeDefinitionId;
    fn as_object(self, ctx: &GraphBuilder<'_>) -> Option<ObjectLikeTarget<Self>>;
    fn on_missing_field(self, ctx: &GraphBuilder<'_>) -> OnMissingField;
}

pub(super) struct ObjectLikeTarget<T> {
    pub is_one_of: bool,
    pub target_fields: Vec<(StringId, (T, Wrapping))>,
}

pub(super) enum OnMissingField {
    None,
    DefaultValue(SchemaInputValueId),
    Providable,
}

impl TargetField for (SubgraphId, InputValueDefinitionId) {
    type Id = InputValueDefinitionId;

    fn id(self) -> Self::Id {
        self.1
    }

    fn id_display<'g>(id: Self::Id, ctx: &'g GraphBuilder<'_>) -> &'g str {
        &ctx[ctx.graph[id].name_id]
    }

    fn display(self, ctx: &GraphBuilder<'_>) -> String {
        let id = self.id();
        match ctx.graph[id].parent_id {
            InputValueParentDefinitionId::Field(field_id) => {
                let field = &ctx.graph[field_id];
                let argument = &ctx.graph[id];
                format!(
                    "{}.{}.{}",
                    ctx[ctx.definition_name_id(field.parent_entity_id.into())],
                    ctx[field.name_id],
                    ctx[argument.name_id]
                )
            }
            InputValueParentDefinitionId::InputObject(input_object_id) => {
                let input_object = &ctx.graph[input_object_id];
                let input_field = &ctx.graph[id];
                format!("{}.{}", &ctx[input_object.name_id], &ctx[input_field.name_id])
            }
        }
    }

    fn type_definition(self, graph: &Graph) -> TypeDefinitionId {
        graph[self.id()].ty_record.definition_id
    }

    fn as_object(self, ctx: &GraphBuilder<'_>) -> Option<ObjectLikeTarget<Self>> {
        let (subgraph_id, input_value_id) = self;
        let input_object_id = ctx.graph[input_value_id].ty_record.definition_id.as_input_object()?;

        let fields = ctx.graph[input_object_id]
            .input_field_ids
            .into_iter()
            .filter(|id| {
                ctx.graph[*id]
                    .is_internal_in_id
                    .is_none_or(|internal_id| internal_id == subgraph_id)
            })
            .map(|id| {
                (
                    ctx.graph[id].name_id,
                    ((subgraph_id, id), ctx.graph[id].ty_record.wrapping),
                )
            })
            .collect::<Vec<_>>();

        Some(ObjectLikeTarget {
            is_one_of: ctx.graph[input_object_id].is_one_of,
            target_fields: fields,
        })
    }

    fn on_missing_field(self, ctx: &GraphBuilder<'_>) -> OnMissingField {
        match ctx.graph[self.id()].default_value_id {
            Some(default_value_id) => OnMissingField::DefaultValue(default_value_id),
            None => OnMissingField::None,
        }
    }
}

impl TargetField for (SubgraphId, FieldDefinitionId) {
    type Id = FieldDefinitionId;

    fn id(self) -> Self::Id {
        self.1
    }

    fn id_display<'g>(id: Self::Id, ctx: &'g GraphBuilder<'_>) -> &'g str {
        &ctx[ctx.graph[id].name_id]
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

    fn as_object(self, ctx: &GraphBuilder<'_>) -> Option<ObjectLikeTarget<Self>> {
        let (subgraph_id, field_id) = self;
        let entity_id = ctx.graph[field_id].ty_record.definition_id.as_entity()?;

        let field_ids = match entity_id {
            EntityDefinitionId::Interface(id) => ctx.graph[id].field_ids,
            EntityDefinitionId::Object(id) => ctx.graph[id].field_ids,
        };
        let fields = field_ids
            .into_iter()
            .filter(|id| ctx.graph[*id].exists_in_subgraph_ids.contains(&subgraph_id))
            .map(|id| {
                (
                    ctx.graph[id].name_id,
                    ((subgraph_id, id), ctx.graph[id].ty_record.wrapping),
                )
            })
            .collect::<Vec<_>>();

        Some(ObjectLikeTarget {
            is_one_of: false,
            target_fields: fields,
        })
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
