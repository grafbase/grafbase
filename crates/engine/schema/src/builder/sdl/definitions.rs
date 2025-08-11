use cynic_parser::type_system::DirectiveLocation;

use crate::{
    EntityDefinitionId, EnumDefinitionId, EnumValueId, FieldDefinitionId, InputObjectDefinitionId,
    InputValueDefinitionId, InterfaceDefinitionId, ObjectDefinitionId, ScalarDefinitionId, SubgraphId,
    UnionDefinitionId, builder::GraphBuilder,
};

use super::*;

#[derive(Copy, Clone)]
pub(crate) enum SdlDefinition<'a> {
    SchemaDirective(SubgraphId),
    Scalar(ScalarSdlDefinition<'a>),
    Object(ObjectSdlDefinition<'a>),
    Interface(InterfaceSdlDefinition<'a>),
    Union(UnionSdlDefinition<'a>),
    Enum(EnumSdlDefinition<'a>),
    InputObject(InputObjectSdlDefinition<'a>),
    FieldDefinition(FieldSdlDefinition<'a>),
    InputFieldDefinition(InputFieldSdlDefinition<'a>),
    ArgumentDefinition(ArgumentSdlDefinition<'a>),
    EnumValue(EnumValueSdlDefinition<'a>),
}

impl<'a> SdlDefinition<'a> {
    pub(crate) fn to_site_string(self, builder: &GraphBuilder<'_>) -> String {
        match self {
            Self::Scalar(def) => def.to_site_string(builder),
            Self::Object(def) => def.to_site_string(builder),
            Self::Interface(def) => def.to_site_string(builder),
            Self::Union(def) => def.to_site_string(builder),
            Self::Enum(def) => def.to_site_string(builder),
            Self::InputObject(def) => def.to_site_string(builder),
            Self::FieldDefinition(def) => def.to_site_string(builder),
            Self::InputFieldDefinition(def) => def.to_site_string(builder),
            Self::ArgumentDefinition(def) => def.to_site_string(builder),
            Self::EnumValue(def) => def.to_site_string(builder),
            Self::SchemaDirective(id) => {
                let name = match id {
                    SubgraphId::Graphql(id) => &builder.ctx[builder.ctx[id].name_id],
                    SubgraphId::Introspection => "Introspection",
                    SubgraphId::Virtual(id) => &builder.ctx[builder.ctx[id].name_id],
                };
                format!("subgraph named '{name}'")
            }
        }
    }

    pub(crate) fn location(self) -> DirectiveLocation {
        match self {
            Self::Scalar(_) => ScalarSdlDefinition::location(),
            Self::Object(_) => ObjectSdlDefinition::location(),
            Self::Interface(_) => InterfaceSdlDefinition::location(),
            Self::Union(_) => UnionSdlDefinition::location(),
            Self::Enum(_) => EnumSdlDefinition::location(),
            Self::InputObject(_) => InputObjectSdlDefinition::location(),
            Self::FieldDefinition(_) => FieldSdlDefinition::location(),
            Self::InputFieldDefinition(_) => InputFieldSdlDefinition::location(),
            Self::ArgumentDefinition(_) => ArgumentSdlDefinition::location(),
            Self::EnumValue(_) => EnumValueSdlDefinition::location(),
            Self::SchemaDirective(_) => DirectiveLocation::Schema,
        }
    }

    pub fn as_type(self) -> Option<SdlTypeDefinition<'a>> {
        match self {
            Self::Scalar(def) => Some(SdlTypeDefinition::Scalar(def)),
            Self::Object(def) => Some(SdlTypeDefinition::Object(def)),
            Self::Interface(def) => Some(SdlTypeDefinition::Interface(def)),
            Self::Union(def) => Some(SdlTypeDefinition::Union(def)),
            Self::Enum(def) => Some(SdlTypeDefinition::Enum(def)),
            Self::InputObject(def) => Some(SdlTypeDefinition::InputObject(def)),
            _ => None,
        }
    }

    pub fn as_entity(self) -> Option<EntitySdlDefinition<'a>> {
        match self {
            Self::Object(def) => Some(EntitySdlDefinition::Object(def)),
            Self::Interface(def) => Some(EntitySdlDefinition::Interface(def)),
            _ => None,
        }
    }

    pub fn as_field(self) -> Option<FieldSdlDefinition<'a>> {
        match self {
            Self::FieldDefinition(def) => Some(def),
            _ => None,
        }
    }

    pub fn span(self) -> Span {
        match self {
            Self::Scalar(def) => def.span(),
            Self::Object(def) => def.span(),
            Self::Interface(def) => def.span(),
            Self::Union(def) => def.span(),
            Self::Enum(def) => def.span(),
            Self::InputObject(def) => def.span(),
            Self::FieldDefinition(def) => def.span(),
            Self::InputFieldDefinition(def) => def.span(),
            Self::ArgumentDefinition(def) => def.span(),
            Self::EnumValue(def) => def.span(),
            Self::SchemaDirective(_) => Span::default(),
        }
    }

    pub fn directives(self) -> Iter<'a, Directive<'a>> {
        match self {
            Self::Scalar(def) => def.directives(),
            Self::Object(def) => def.directives(),
            Self::Interface(def) => def.directives(),
            Self::Union(def) => def.directives(),
            Self::Enum(def) => def.directives(),
            Self::InputObject(def) => def.directives(),
            Self::FieldDefinition(def) => def.directives(),
            Self::InputFieldDefinition(def) => def.directives(),
            Self::ArgumentDefinition(def) => def.directives(),
            Self::EnumValue(def) => def.directives(),
            Self::SchemaDirective(_) => unreachable!(),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum SdlTypeDefinition<'a> {
    Scalar(ScalarSdlDefinition<'a>),
    Object(ObjectSdlDefinition<'a>),
    Interface(InterfaceSdlDefinition<'a>),
    Union(UnionSdlDefinition<'a>),
    Enum(EnumSdlDefinition<'a>),
    InputObject(InputObjectSdlDefinition<'a>),
}

impl<'a> SdlTypeDefinition<'a> {
    pub(crate) fn name(&self) -> &'a str {
        match self {
            Self::Scalar(def) => def.name(),
            Self::Object(def) => def.name(),
            Self::Interface(def) => def.name(),
            Self::Union(def) => def.name(),
            Self::Enum(def) => def.name(),
            Self::InputObject(def) => def.name(),
        }
    }

    #[allow(unused)]
    pub(crate) fn directives(self) -> Iter<'a, Directive<'a>> {
        match self {
            Self::Scalar(def) => def.directives(),
            Self::Object(def) => def.directives(),
            Self::Interface(def) => def.directives(),
            Self::Union(def) => def.directives(),
            Self::Enum(def) => def.directives(),
            Self::InputObject(def) => def.directives(),
        }
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Scalar(def) => def.span(),
            Self::Object(def) => def.span(),
            Self::Interface(def) => def.span(),
            Self::Union(def) => def.span(),
            Self::Enum(def) => def.span(),
            Self::InputObject(def) => def.span(),
        }
    }

    #[allow(unused)]
    pub(crate) fn to_site_string(self, builder: &GraphBuilder<'_>) -> String {
        match self {
            Self::Scalar(def) => def.to_site_string(builder),
            Self::Object(def) => def.to_site_string(builder),
            Self::Interface(def) => def.to_site_string(builder),
            Self::Union(def) => def.to_site_string(builder),
            Self::Enum(def) => def.to_site_string(builder),
            Self::InputObject(def) => def.to_site_string(builder),
        }
    }

    #[allow(unused)]
    pub(crate) fn location(self) -> DirectiveLocation {
        match self {
            Self::Scalar(_) => ScalarSdlDefinition::location(),
            Self::Object(_) => ObjectSdlDefinition::location(),
            Self::Interface(_) => InterfaceSdlDefinition::location(),
            Self::Union(_) => UnionSdlDefinition::location(),
            Self::Enum(_) => EnumSdlDefinition::location(),
            Self::InputObject(_) => InputObjectSdlDefinition::location(),
        }
    }
}

impl<'a> From<SdlTypeDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: SdlTypeDefinition<'a>) -> Self {
        match def {
            SdlTypeDefinition::Scalar(def) => SdlDefinition::Scalar(def),
            SdlTypeDefinition::Object(def) => SdlDefinition::Object(def),
            SdlTypeDefinition::Interface(def) => SdlDefinition::Interface(def),
            SdlTypeDefinition::Union(def) => SdlDefinition::Union(def),
            SdlTypeDefinition::Enum(def) => SdlDefinition::Enum(def),
            SdlTypeDefinition::InputObject(def) => SdlDefinition::InputObject(def),
        }
    }
}

impl<'a> From<ScalarSdlDefinition<'a>> for SdlTypeDefinition<'a> {
    fn from(def: ScalarSdlDefinition<'a>) -> Self {
        SdlTypeDefinition::Scalar(def)
    }
}

impl<'a> From<ObjectSdlDefinition<'a>> for SdlTypeDefinition<'a> {
    fn from(def: ObjectSdlDefinition<'a>) -> Self {
        SdlTypeDefinition::Object(def)
    }
}

impl<'a> From<InterfaceSdlDefinition<'a>> for SdlTypeDefinition<'a> {
    fn from(def: InterfaceSdlDefinition<'a>) -> Self {
        SdlTypeDefinition::Interface(def)
    }
}

impl<'a> From<UnionSdlDefinition<'a>> for SdlTypeDefinition<'a> {
    fn from(def: UnionSdlDefinition<'a>) -> Self {
        SdlTypeDefinition::Union(def)
    }
}

impl<'a> From<EnumSdlDefinition<'a>> for SdlTypeDefinition<'a> {
    fn from(def: EnumSdlDefinition<'a>) -> Self {
        SdlTypeDefinition::Enum(def)
    }
}

impl<'a> From<InputObjectSdlDefinition<'a>> for SdlTypeDefinition<'a> {
    fn from(def: InputObjectSdlDefinition<'a>) -> Self {
        SdlTypeDefinition::InputObject(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) enum EntitySdlDefinition<'a> {
    Object(ObjectSdlDefinition<'a>),
    Interface(InterfaceSdlDefinition<'a>),
}

impl<'a> EntitySdlDefinition<'a> {
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        match self {
            Self::Object(def) => def.directives(),
            Self::Interface(def) => def.directives(),
        }
    }

    pub fn name(&self) -> &'a str {
        match self {
            Self::Object(def) => def.name(),
            Self::Interface(def) => def.name(),
        }
    }

    pub fn id(&self) -> EntityDefinitionId {
        match self {
            Self::Object(def) => EntityDefinitionId::Object(def.id),
            Self::Interface(def) => EntityDefinitionId::Interface(def.id),
        }
    }

    #[allow(unused)]
    pub fn span(&self) -> Span {
        match self {
            Self::Object(def) => def.span(),
            Self::Interface(def) => def.span(),
        }
    }

    pub fn to_site_string(self, builder: &GraphBuilder<'_>) -> String {
        match self {
            Self::Object(def) => def.to_site_string(builder),
            Self::Interface(def) => def.to_site_string(builder),
        }
    }
}

impl<'a> From<ObjectSdlDefinition<'a>> for EntitySdlDefinition<'a> {
    fn from(def: ObjectSdlDefinition<'a>) -> Self {
        EntitySdlDefinition::Object(def)
    }
}

impl<'a> From<InterfaceSdlDefinition<'a>> for EntitySdlDefinition<'a> {
    fn from(def: InterfaceSdlDefinition<'a>) -> Self {
        EntitySdlDefinition::Interface(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) enum InputValueSdlDefinition<'a> {
    InputField(InputFieldSdlDefinition<'a>),
    Argument(ArgumentSdlDefinition<'a>),
}

impl InputValueSdlDefinition<'_> {
    pub fn to_site_string(self, builder: &GraphBuilder<'_>) -> String {
        match self {
            Self::InputField(def) => def.to_site_string(builder),
            Self::Argument(def) => def.to_site_string(builder),
        }
    }
}

impl<'a> From<InputFieldSdlDefinition<'a>> for InputValueSdlDefinition<'a> {
    fn from(def: InputFieldSdlDefinition<'a>) -> Self {
        Self::InputField(def)
    }
}

impl<'a> From<ArgumentSdlDefinition<'a>> for InputValueSdlDefinition<'a> {
    fn from(def: ArgumentSdlDefinition<'a>) -> Self {
        Self::Argument(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ScalarSdlDefinition<'a> {
    pub id: ScalarDefinitionId,
    pub definition: ScalarDefinition<'a>,
}

impl<'a> std::ops::Deref for ScalarSdlDefinition<'a> {
    type Target = ScalarDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl ScalarSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        ctx[graph[self.id].name_id].clone()
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::Scalar
    }
}

impl<'a> From<ScalarSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: ScalarSdlDefinition<'a>) -> Self {
        SdlDefinition::Scalar(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ObjectSdlDefinition<'a> {
    pub id: ObjectDefinitionId,
    pub definition: ObjectDefinition<'a>,
}

impl<'a> std::ops::Deref for ObjectSdlDefinition<'a> {
    type Target = ObjectDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl ObjectSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        ctx[graph[self.id].name_id].clone()
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::Object
    }
}

impl<'a> From<ObjectSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: ObjectSdlDefinition<'a>) -> Self {
        SdlDefinition::Object(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct InterfaceSdlDefinition<'a> {
    pub id: InterfaceDefinitionId,
    pub definition: InterfaceDefinition<'a>,
}

impl<'a> std::ops::Deref for InterfaceSdlDefinition<'a> {
    type Target = InterfaceDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl InterfaceSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        ctx[graph[self.id].name_id].clone()
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::Interface
    }
}

impl<'a> From<InterfaceSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: InterfaceSdlDefinition<'a>) -> Self {
        SdlDefinition::Interface(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct UnionSdlDefinition<'a> {
    pub id: UnionDefinitionId,
    pub definition: UnionDefinition<'a>,
}

impl<'a> std::ops::Deref for UnionSdlDefinition<'a> {
    type Target = UnionDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl UnionSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        ctx[graph[self.id].name_id].clone()
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::Union
    }
}

impl<'a> From<UnionSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: UnionSdlDefinition<'a>) -> Self {
        SdlDefinition::Union(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct EnumSdlDefinition<'a> {
    pub id: EnumDefinitionId,
    pub definition: EnumDefinition<'a>,
}

impl<'a> std::ops::Deref for EnumSdlDefinition<'a> {
    type Target = EnumDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl EnumSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        ctx[graph[self.id].name_id].clone()
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::Enum
    }
}

impl<'a> From<EnumSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: EnumSdlDefinition<'a>) -> Self {
        SdlDefinition::Enum(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct InputObjectSdlDefinition<'a> {
    pub id: InputObjectDefinitionId,
    pub definition: InputObjectDefinition<'a>,
}

impl<'a> std::ops::Deref for InputObjectSdlDefinition<'a> {
    type Target = InputObjectDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl InputObjectSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        ctx[graph[self.id].name_id].clone()
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::InputObject
    }
}

impl<'a> From<InputObjectSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: InputObjectSdlDefinition<'a>) -> Self {
        SdlDefinition::InputObject(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct FieldSdlDefinition<'a> {
    pub id: FieldDefinitionId,
    pub parent: TypeDefinition<'a>,
    pub definition: FieldDefinition<'a>,
}

impl<'a> std::ops::Deref for FieldSdlDefinition<'a> {
    type Target = FieldDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl FieldSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        let field = &graph[self.id];
        let parent_name_id = match field.parent_entity_id {
            EntityDefinitionId::Interface(id) => graph[id].name_id,
            EntityDefinitionId::Object(id) => graph[id].name_id,
        };
        format!("{}.{}", ctx[parent_name_id], ctx[field.name_id])
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::FieldDefinition
    }
}

impl<'a> From<FieldSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: FieldSdlDefinition<'a>) -> Self {
        SdlDefinition::FieldDefinition(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct InputFieldSdlDefinition<'a> {
    pub parent_id: InputObjectDefinitionId,
    pub id: InputValueDefinitionId,
    pub definition: InputValueDefinition<'a>,
}

impl<'a> std::ops::Deref for InputFieldSdlDefinition<'a> {
    type Target = InputValueDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl InputFieldSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        format!("{}.{}", ctx[graph[self.parent_id].name_id], ctx[graph[self.id].name_id])
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::InputFieldDefinition
    }
}

impl<'a> From<InputFieldSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: InputFieldSdlDefinition<'a>) -> Self {
        SdlDefinition::InputFieldDefinition(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ArgumentSdlDefinition<'a> {
    pub field_id: FieldDefinitionId,
    pub id: InputValueDefinitionId,
    pub definition: InputValueDefinition<'a>,
}

impl<'a> std::ops::Deref for ArgumentSdlDefinition<'a> {
    type Target = InputValueDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl ArgumentSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        let field = &graph[self.field_id];
        let parent_name_id = match field.parent_entity_id {
            EntityDefinitionId::Interface(id) => graph[id].name_id,
            EntityDefinitionId::Object(id) => graph[id].name_id,
        };
        format!(
            "{}.{}.{}",
            ctx[parent_name_id], ctx[field.name_id], ctx[graph[self.id].name_id]
        )
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::ArgumentDefinition
    }
}

impl<'a> From<ArgumentSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: ArgumentSdlDefinition<'a>) -> Self {
        SdlDefinition::ArgumentDefinition(def)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct EnumValueSdlDefinition<'a> {
    pub id: EnumValueId,
    pub definition: EnumValueDefinition<'a>,
}

impl<'a> std::ops::Deref for EnumValueSdlDefinition<'a> {
    type Target = EnumValueDefinition<'a>;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl EnumValueSdlDefinition<'_> {
    pub fn to_site_string(self, GraphBuilder { ctx, graph, .. }: &GraphBuilder<'_>) -> String {
        let enum_id = graph[self.id].parent_enum_id;
        format!("{}.{}", ctx[graph[enum_id].name_id], ctx[graph[self.id].name_id])
    }

    pub fn location() -> DirectiveLocation {
        DirectiveLocation::EnumValue
    }
}

impl<'a> From<EnumValueSdlDefinition<'a>> for SdlDefinition<'a> {
    fn from(def: EnumValueSdlDefinition<'a>) -> Self {
        SdlDefinition::EnumValue(def)
    }
}

impl From<SubgraphId> for SdlDefinition<'_> {
    fn from(id: SubgraphId) -> Self {
        SdlDefinition::SchemaDirective(id)
    }
}
