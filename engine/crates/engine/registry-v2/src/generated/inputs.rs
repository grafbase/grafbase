use super::prelude::ids::{InputObjectTypeId, InputValidatorId, MetaInputValueId};
use super::prelude::*;
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct MetaInputValueRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2")]
    pub ty: MetaInputValueTypeRecord,
    #[serde(rename = "3", skip_serializing_if = "Option::is_none", default)]
    pub default_value: Option<Box<ConstValue>>,
    #[serde(rename = "4", skip_serializing_if = "Option::is_none", default)]
    pub rename: Option<StringId>,
    #[serde(rename = "5", skip_serializing_if = "crate::Container::is_empty", default)]
    pub validators: IdRange<InputValidatorId>,
}

#[derive(Clone, Copy)]
pub struct MetaInputValue<'a>(pub(crate) ReadContext<'a, MetaInputValueId>);

impl<'a> MetaInputValue<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn default_value(&self) -> Option<&'a ConstValue> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).default_value.as_deref()
    }
    pub fn rename(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).rename.map(|id| registry.lookup(id))
    }
    pub fn validators(&self) -> Iter<'a, InputValidator<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).validators, registry)
    }
}

impl fmt::Debug for MetaInputValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaInputValue")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("ty", &self.ty())
            .field("default_value", &self.default_value())
            .field("rename", &self.rename())
            .field("validators", &self.validators().collect::<Vec<_>>())
            .finish()
    }
}

impl RegistryId for MetaInputValueId {
    type Reader<'a> = MetaInputValue<'a>;
}

impl IdReader for MetaInputValue<'_> {
    type Id = MetaInputValueId;
}

impl<'a> From<ReadContext<'a, MetaInputValueId>> for MetaInputValue<'a> {
    fn from(value: ReadContext<'a, MetaInputValueId>) -> Self {
        Self(value)
    }
}

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct InputObjectTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "crate::Container::is_empty", default)]
    pub input_fields: IdRange<MetaInputValueId>,
    #[serde(rename = "3", skip_serializing_if = "crate::is_false", default)]
    pub oneof: bool,
}

#[derive(Clone, Copy)]
pub struct InputObjectType<'a>(pub(crate) ReadContext<'a, InputObjectTypeId>);

impl<'a> InputObjectType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn input_fields(&self) -> Iter<'a, MetaInputValue<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).input_fields, registry)
    }
    pub fn oneof(&self) -> bool {
        let registry = self.0.registry;
        registry.lookup(self.0.id).oneof
    }
}

impl fmt::Debug for InputObjectType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputObjectType")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("input_fields", &self.input_fields().collect::<Vec<_>>())
            .field("oneof", &self.oneof())
            .finish()
    }
}

impl std::cmp::PartialEq for InputObjectType<'_> {
    fn eq(&self, other: &InputObjectType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for InputObjectType<'_> {}

impl RegistryId for InputObjectTypeId {
    type Reader<'a> = InputObjectType<'a>;
}

impl IdReader for InputObjectType<'_> {
    type Id = InputObjectTypeId;
}

impl<'a> From<ReadContext<'a, InputObjectTypeId>> for InputObjectType<'a> {
    fn from(value: ReadContext<'a, InputObjectTypeId>) -> Self {
        Self(value)
    }
}

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct InputValidatorRecord {
    #[serde(rename = "0")]
    pub validator: DynValidator,
}

#[derive(Clone, Copy)]
pub struct InputValidator<'a>(pub(crate) ReadContext<'a, InputValidatorId>);

impl<'a> InputValidator<'a> {
    pub fn validator(&self) -> &'a DynValidator {
        let registry = self.0.registry;
        &registry.lookup(self.0.id).validator
    }
}

impl fmt::Debug for InputValidator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputValidator")
            .field("validator", &self.validator())
            .finish()
    }
}

impl RegistryId for InputValidatorId {
    type Reader<'a> = InputValidator<'a>;
}

impl IdReader for InputValidator<'_> {
    type Id = InputValidatorId;
}

impl<'a> From<ReadContext<'a, InputValidatorId>> for InputValidator<'a> {
    fn from(value: ReadContext<'a, InputValidatorId>) -> Self {
        Self(value)
    }
}
