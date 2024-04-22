//! GraphQL has a bunch of different kinds of types.
//!
//! In our system these are represented by the `MetaType` enum and the structs it contains.
//! But there are not many positions in GraphQL where all kinds are valid.  Selection fields
//! can only be output types, input fields can only be input types etc. etc.
//!
//! The enums in this file provide alternative groupings for all of these different cases.

use std::iter;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use registry_v2::{MetaInputValue, MetaType, ObjectType};

use crate::Error;

/// The kinds of types we work with in GraphQL
///
/// This enum is composed of both the individual kinds of MetaType and some
/// additional variants for the different sub-groups we work with.
///
/// It's mostly used in error messages at the moment, but could be used for
/// other things.
#[derive(Debug)]
pub enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    SelectionSetTarget,
    OutputType,
    InputType,
}

impl TypeKind {
    pub fn for_metatype(metatype: registry_v2::MetaType) -> Self {
        match metatype {
            MetaType::Object(_) => TypeKind::Object,
            MetaType::Interface(_) => TypeKind::Interface,
            MetaType::Union(_) => TypeKind::Union,
            MetaType::Enum(_) => TypeKind::Enum,
            MetaType::InputObject(_) => TypeKind::InputObject,
            MetaType::Scalar(_) => TypeKind::Scalar,
        }
    }
}

pub(super) trait MetaTypeExt {
    fn kind(&self) -> TypeKind;
}

impl MetaTypeExt for MetaType<'_> {
    fn kind(&self) -> TypeKind {
        match self {
            MetaType::Scalar(_) => TypeKind::Scalar,
            MetaType::Object(_) => TypeKind::Object,
            MetaType::Interface { .. } => TypeKind::Interface,
            MetaType::Union { .. } => TypeKind::Union,
            MetaType::Enum { .. } => TypeKind::Enum,
            MetaType::InputObject { .. } => TypeKind::InputObject,
        }
    }
}

/// A type in output position - i.e. the type of a field in an Object/Interface/selection set.
#[derive(Clone, Copy, Debug)]
pub enum OutputType<'a> {
    Scalar(registry_v2::ScalarType<'a>),
    Object(registry_v2::ObjectType<'a>),
    Interface(registry_v2::InterfaceType<'a>),
    Union(registry_v2::UnionType<'a>),
    Enum(registry_v2::EnumType<'a>),
}

impl<'a> OutputType<'a> {
    pub fn name(&self) -> &str {
        match self {
            OutputType::Scalar(scalar) => scalar.name(),
            OutputType::Object(object) => object.name(),
            OutputType::Interface(interface) => interface.name(),
            OutputType::Union(union) => union.name(),
            OutputType::Enum(en) => en.name(),
        }
    }

    pub fn field(&self, name: &str) -> Option<registry_v2::MetaField<'a>> {
        match self {
            OutputType::Object(object) => object.field(name),
            OutputType::Interface(interface) => interface.field(name),
            _ => None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, OutputType::Scalar(_) | OutputType::Enum(_))
    }

    pub fn fields(&self) -> Box<dyn Iterator<Item = registry_v2::MetaField<'a>> + '_> {
        match self {
            OutputType::Scalar(_) | OutputType::Union(_) | OutputType::Enum(_) => Box::new(iter::empty()),
            OutputType::Object(object) => Box::new(object.fields()),
            OutputType::Interface(interface) => Box::new(interface.fields()),
        }
    }

    pub fn object(&self) -> Option<registry_v2::ObjectType<'a>> {
        match self {
            OutputType::Object(obj) => Some(*obj),
            _ => None,
        }
    }

    pub fn kind(&self) -> TypeKind {
        match self {
            OutputType::Object(_) => TypeKind::Object,
            OutputType::Interface(_) => TypeKind::Interface,
            OutputType::Union(_) => TypeKind::Union,
            OutputType::Scalar(_) => TypeKind::Scalar,
            OutputType::Enum(_) => TypeKind::Enum,
        }
    }
}

impl<'a> TryFrom<registry_v2::MetaType<'a>> for OutputType<'a> {
    type Error = Error;

    fn try_from(value: registry_v2::MetaType<'a>) -> Result<Self, Self::Error> {
        match value {
            registry_v2::MetaType::Scalar(scalar) => Ok(OutputType::Scalar(scalar)),
            registry_v2::MetaType::Object(object) => Ok(OutputType::Object(object)),
            registry_v2::MetaType::Interface(interface) => Ok(OutputType::Interface(interface)),
            registry_v2::MetaType::Union(union) => Ok(OutputType::Union(union)),
            registry_v2::MetaType::Enum(en) => Ok(OutputType::Enum(en)),
            registry_v2::MetaType::InputObject(_) => {
                Err(Error::unexpected_kind(value.name(), value.kind(), TypeKind::OutputType))
            }
        }
    }
}

impl<'a> From<OutputType<'a>> for registry_v2::MetaType<'a> {
    fn from(value: OutputType<'a>) -> Self {
        match value {
            OutputType::Scalar(inner) => MetaType::Scalar(inner),
            OutputType::Object(inner) => MetaType::Object(inner),
            OutputType::Interface(inner) => MetaType::Interface(inner),
            OutputType::Union(inner) => MetaType::Union(inner),
            OutputType::Enum(inner) => MetaType::Enum(inner),
        }
    }
}

/// A type in output position - i.e. an argument or field of an input object
#[derive(Debug, Clone, Copy)]
pub enum InputType<'a> {
    Scalar(registry_v2::ScalarType<'a>),
    Enum(registry_v2::EnumType<'a>),
    InputObject(registry_v2::InputObjectType<'a>),
}

impl<'a> InputType<'a> {
    pub fn name(&self) -> &str {
        match self {
            InputType::Scalar(scalar) => scalar.name(),
            InputType::Enum(en) => en.name(),
            InputType::InputObject(input_object) => input_object.name(),
        }
    }

    pub fn field(&self, name: &str) -> Option<MetaInputValue<'a>> {
        match self {
            InputType::Scalar(_) | InputType::Enum(_) => None,
            InputType::InputObject(input_object) => input_object.field(name),
        }
    }

    pub fn fields(&self) -> Box<dyn ExactSizeIterator<Item = registry_v2::MetaInputValue<'a>> + 'a> {
        match self {
            InputType::Scalar(_) | InputType::Enum(_) => Box::new(iter::empty()),
            InputType::InputObject(input_object) => Box::new(input_object.input_fields()),
        }
    }

    pub fn is_input_object(&self) -> bool {
        matches!(self, InputType::InputObject(_))
    }

    pub fn as_input_object(&self) -> Option<registry_v2::InputObjectType<'a>> {
        match self {
            InputType::InputObject(obj) => Some(*obj),
            _ => None,
        }
    }
}

impl<'a> TryFrom<registry_v2::MetaType<'a>> for InputType<'a> {
    type Error = Error;

    fn try_from(value: registry_v2::MetaType<'a>) -> Result<Self, Self::Error> {
        match value {
            registry_v2::MetaType::Scalar(scalar) => Ok(InputType::Scalar(scalar)),
            registry_v2::MetaType::Enum(en) => Ok(InputType::Enum(en)),
            registry_v2::MetaType::InputObject(object) => Ok(InputType::InputObject(object)),
            _ => Err(Error::unexpected_kind(value.name(), value.kind(), TypeKind::InputType)),
        }
    }
}

/// A reference to a MetaType in a selection set context.
///
/// When we're processing a selection set in GQL we know that the target
/// of the selection set is one of the composite output types. This enum
/// lets us work with just those types rather than having to work with
/// all the MetaType variants.
#[derive(Clone, Copy, Debug)]
pub enum SelectionSetTarget<'a> {
    Object(registry_v2::ObjectType<'a>),
    Interface(registry_v2::InterfaceType<'a>),
    Union(registry_v2::UnionType<'a>),
}

impl<'a> SelectionSetTarget<'a> {
    pub fn name(&self) -> &str {
        match self {
            SelectionSetTarget::Object(object) => object.name(),
            SelectionSetTarget::Interface(interface) => interface.name(),
            SelectionSetTarget::Union(union) => union.name(),
        }
    }

    pub fn field(&self, name: &str) -> Option<registry_v2::MetaField<'a>> {
        match self {
            SelectionSetTarget::Object(obj) => obj.field(name),
            SelectionSetTarget::Interface(iface) => iface.field(name),
            SelectionSetTarget::Union(union) => union.field(name),
        }
    }
}

impl<'a> TryFrom<registry_v2::MetaType<'a>> for SelectionSetTarget<'a> {
    type Error = Error;

    fn try_from(value: registry_v2::MetaType<'a>) -> Result<Self, Self::Error> {
        match value {
            MetaType::Object(object) => Ok(SelectionSetTarget::Object(object)),
            MetaType::Interface(interface) => Ok(SelectionSetTarget::Interface(interface)),
            MetaType::Union(union) => Ok(SelectionSetTarget::Union(union)),
            _ => Err(Error::unexpected_kind(
                value.name(),
                value.kind(),
                TypeKind::SelectionSetTarget,
            )),
        }
    }
}

impl<'a> TryFrom<OutputType<'a>> for SelectionSetTarget<'a> {
    type Error = Error;

    fn try_from(value: OutputType<'a>) -> Result<Self, Self::Error> {
        match value {
            OutputType::Object(object) => Ok(SelectionSetTarget::Object(object)),
            OutputType::Interface(interface) => Ok(SelectionSetTarget::Interface(interface)),
            OutputType::Union(union) => Ok(SelectionSetTarget::Union(union)),
            _ => Err(Error::unexpected_kind(
                value.name(),
                value.kind(),
                TypeKind::SelectionSetTarget,
            )),
        }
    }
}
