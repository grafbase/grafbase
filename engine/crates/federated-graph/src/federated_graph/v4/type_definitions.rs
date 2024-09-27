use super::{Directives, FederatedGraph, StringId, TypeDefinitionId};

pub type TypeDefinition<'a> = super::view::ViewNested<'a, TypeDefinitionId, TypeDefinitionRecord>;

#[derive(Clone, Debug)]
pub struct TypeDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringId>,
    pub directives: Directives,
    pub kind: TypeDefinitionKind,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeDefinitionKind {
    Object,
    Interface,
    Enum,
    Union,
    Scalar,
    InputObject,
}

impl TypeDefinitionKind {
    /// Returns `true` if the type definition kind is [`Scalar`].
    ///
    /// [`Scalar`]: TypeDefinitionKind::Scalar
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar)
    }

    /// Returns `true` if the type definition kind is [`Enum`].
    ///
    /// [`Enum`]: TypeDefinitionKind::Enum
    #[must_use]
    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum)
    }
}

impl FederatedGraph {
    pub fn push_type_definition(&mut self, type_def: TypeDefinitionRecord) -> TypeDefinitionId {
        let id = self.type_definitions.len().into();
        self.type_definitions.push(type_def);
        id
    }
}
