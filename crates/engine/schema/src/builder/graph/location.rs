use crate::{
    EntityDefinitionId, EnumDefinitionId, EnumValueId, FieldDefinitionId, InputObjectDefinitionId,
    InputValueDefinitionId, InterfaceDefinitionId, ObjectDefinitionId, ScalarDefinitionId, SubgraphId,
    UnionDefinitionId, builder::sdl,
};

use super::GraphBuilder;

#[derive(Copy, Clone)]
pub enum SchemaLocation<'a> {
    SchemaDirective(SubgraphId),
    Scalar(ScalarDefinitionId, sdl::ScalarDefinition<'a>),
    Object(ObjectDefinitionId, sdl::ObjectDefinition<'a>),
    Interface(InterfaceDefinitionId, sdl::InterfaceDefinition<'a>),
    Union(UnionDefinitionId, sdl::UnionDefinition<'a>),
    Enum(EnumDefinitionId, sdl::EnumDefinition<'a>),
    InputObject(InputObjectDefinitionId, sdl::InputObjectDefinition<'a>),
    FieldDefinition(FieldDefinitionId, sdl::TypeDefinition<'a>, sdl::FieldDefinition<'a>),
    InputFieldDefinition(
        InputObjectDefinitionId,
        InputValueDefinitionId,
        sdl::InputValueDefinition<'a>,
    ),
    ArgumentDefinition(FieldDefinitionId, InputValueDefinitionId, sdl::InputValueDefinition<'a>),
    EnumValue(EnumValueId, sdl::EnumValueDefinition<'a>),
}

impl SchemaLocation<'_> {
    pub fn to_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        match self {
            SchemaLocation::Enum(id, _) => ctx[graph[id].name_id].clone(),
            SchemaLocation::InputObject(id, _) => ctx[graph[id].name_id].clone(),
            SchemaLocation::Interface(id, _) => ctx[graph[id].name_id].clone(),
            SchemaLocation::Object(id, _) => ctx[graph[id].name_id].clone(),
            SchemaLocation::Scalar(id, _) => ctx[graph[id].name_id].clone(),
            SchemaLocation::Union(id, _) => ctx[graph[id].name_id].clone(),
            SchemaLocation::FieldDefinition(id, _, _) => {
                let field = &graph[id];
                let parent_name_id = match field.parent_entity_id {
                    EntityDefinitionId::Interface(id) => graph[id].name_id,
                    EntityDefinitionId::Object(id) => graph[id].name_id,
                };
                format!("{}.{}", ctx[parent_name_id], ctx[field.name_id])
            }
            SchemaLocation::InputFieldDefinition(input_object_id, id, _) => {
                format!("{}.{}", ctx[graph[input_object_id].name_id], ctx[graph[id].name_id])
            }
            SchemaLocation::ArgumentDefinition(field_id, id, _) => {
                let field = &graph[field_id];
                let parent_name_id = match field.parent_entity_id {
                    EntityDefinitionId::Interface(id) => graph[id].name_id,
                    EntityDefinitionId::Object(id) => graph[id].name_id,
                };
                format!(
                    "{}.{}.{}",
                    ctx[parent_name_id], ctx[field.name_id], ctx[graph[id].name_id]
                )
            }
            SchemaLocation::EnumValue(id, _) => {
                let enum_id = graph[id].parent_enum_id;
                format!("{}.{}", ctx[graph[enum_id].name_id], ctx[graph[id].name_id])
            }
            SchemaLocation::SchemaDirective(id) => {
                let name = match id {
                    SubgraphId::GraphqlEndpoint(id) => &ctx[ctx[id].subgraph_name_id],
                    SubgraphId::Introspection => "Introspection",
                    SubgraphId::Virtual(id) => &ctx[ctx[id].subgraph_name_id],
                };
                format!("subgraph named '{name}'")
            }
        }
    }

    pub fn as_cynic_location(self) -> cynic_parser::type_system::DirectiveLocation {
        match self {
            SchemaLocation::Enum(_, _) => cynic_parser::type_system::DirectiveLocation::Enum,
            SchemaLocation::InputObject(_, _) => cynic_parser::type_system::DirectiveLocation::InputObject,
            SchemaLocation::Interface(_, _) => cynic_parser::type_system::DirectiveLocation::Interface,
            SchemaLocation::Object(_, _) => cynic_parser::type_system::DirectiveLocation::Object,
            SchemaLocation::Scalar(_, _) => cynic_parser::type_system::DirectiveLocation::Scalar,
            SchemaLocation::Union(_, _) => cynic_parser::type_system::DirectiveLocation::Union,
            SchemaLocation::FieldDefinition(_, _, _) => cynic_parser::type_system::DirectiveLocation::FieldDefinition,
            SchemaLocation::EnumValue(_, _) => cynic_parser::type_system::DirectiveLocation::EnumValue,
            SchemaLocation::SchemaDirective(_) => cynic_parser::type_system::DirectiveLocation::Schema,
            SchemaLocation::ArgumentDefinition(_, _, _) => {
                cynic_parser::type_system::DirectiveLocation::ArgumentDefinition
            }
            SchemaLocation::InputFieldDefinition(_, _, _) => {
                cynic_parser::type_system::DirectiveLocation::InputFieldDefinition
            }
        }
    }
}
