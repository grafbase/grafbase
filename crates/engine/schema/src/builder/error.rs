use super::{
    graph::GraphContext, EntityDefinitionId, EnumDefinitionId, EnumValueId, FieldDefinitionId, InputObjectDefinitionId,
    InputValueDefinitionId, InputValueError, InterfaceDefinitionId, ObjectDefinitionId, ScalarDefinitionId,
    UnionDefinitionId,
};

#[derive(Debug, Copy, Clone)]
pub enum SchemaLocation {
    Scalar(ScalarDefinitionId, federated_graph::ScalarDefinitionId),
    Object(ObjectDefinitionId, federated_graph::ObjectId),
    Interface(InterfaceDefinitionId, federated_graph::InterfaceId),
    Union(UnionDefinitionId, federated_graph::UnionId),
    Enum(EnumDefinitionId, federated_graph::EnumDefinitionId),
    InputObject(InputObjectDefinitionId, federated_graph::InputObjectId),
    Field(FieldDefinitionId, federated_graph::FieldId),
    InputValue(InputValueDefinitionId, federated_graph::InputValueDefinitionId),
    EnumValue(EnumValueId, federated_graph::EnumValueId),
}

impl SchemaLocation {
    pub fn to_string(self, GraphContext { ctx, graph, .. }: &GraphContext<'_>) -> String {
        match self {
            SchemaLocation::Enum(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::InputObject(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Interface(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Object(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Scalar(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Union(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::Field(id, _) => {
                let field = &graph[id];
                let parent_name_id = match field.parent_entity_id {
                    EntityDefinitionId::Interface(id) => graph[id].name_id,
                    EntityDefinitionId::Object(id) => graph[id].name_id,
                };
                format!("{}.{}", ctx.strings[parent_name_id], ctx.strings[field.name_id])
            }
            SchemaLocation::EnumValue(id, _) => ctx.strings[graph[id].name_id].clone(),
            SchemaLocation::InputValue(id, _) => ctx.strings[graph[id].name_id].clone(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("Invalid URL '{url}': {err}")]
    InvalidUrl { url: String, err: String },
    #[error("At {location}, a required field argument is invalid: {err}")]
    RequiredFieldArgumentCoercionError { location: String, err: InputValueError },
    #[error("An input value named '{name}' has an invalid default value: {err}")]
    DefaultValueCoercionError { name: String, err: InputValueError },
    #[error(transparent)]
    GraphFromSdlError(#[from] federated_graph::DomainError),
    #[error("Unsupported extension: {id}")]
    UnsupportedExtension { id: Box<extension_catalog::Id> },
    #[error("Could not load extension at '{url}': {err}")]
    CouldNotLoadExtension { url: String, err: String },
}
