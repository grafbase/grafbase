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

use super::{
    EnumType, InputObjectType, InterfaceType, MetaField, MetaInputValue, MetaType, ObjectType, ScalarType, UnionType,
};
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

impl MetaType {
    pub(super) fn kind(&self) -> TypeKind {
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
    Scalar(&'a ScalarType),
    Object(&'a ObjectType),
    Interface(&'a InterfaceType),
    Union(&'a UnionType),
    Enum(&'a EnumType),
}

impl<'a> OutputType<'a> {
    pub fn name(&self) -> &str {
        match self {
            OutputType::Scalar(scalar) => &scalar.name,
            OutputType::Object(object) => &object.name,
            OutputType::Interface(interface) => &interface.name,
            OutputType::Union(union) => &union.name,
            OutputType::Enum(en) => &en.name,
        }
    }

    pub fn field(&self, name: &str) -> Option<&MetaField> {
        match self {
            OutputType::Object(object) => object.field_by_name(name),
            OutputType::Interface(interface) => interface.field_by_name(name),
            _ => None,
        }
    }

    pub fn field_map(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            OutputType::Object(object) => Some(&object.fields),
            OutputType::Interface(interface) => Some(&interface.fields),
            _ => None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, OutputType::Scalar(_) | OutputType::Enum(_))
    }

    pub fn fields(&self) -> Box<dyn Iterator<Item = &MetaField> + '_> {
        match self {
            OutputType::Scalar(_) | OutputType::Union(_) | OutputType::Enum(_) => Box::new(iter::empty()),
            OutputType::Object(ObjectType { fields, .. }) | OutputType::Interface(InterfaceType { fields, .. }) => {
                Box::new(fields.values())
            }
        }
    }

    pub fn object(&self) -> Option<&'a ObjectType> {
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

impl<'a> TryFrom<&'a MetaType> for OutputType<'a> {
    type Error = Error;

    fn try_from(value: &'a MetaType) -> Result<Self, Self::Error> {
        match value {
            MetaType::Scalar(scalar) => Ok(OutputType::Scalar(scalar)),
            MetaType::Object(object) => Ok(OutputType::Object(object)),
            MetaType::Interface(interface) => Ok(OutputType::Interface(interface)),
            MetaType::Union(union) => Ok(OutputType::Union(union)),
            MetaType::Enum(en) => Ok(OutputType::Enum(en)),
            MetaType::InputObject(_) => Err(Error::unexpected_kind(value.name(), value.kind(), TypeKind::OutputType)),
        }
    }
}

/// A type in output position - i.e. an argument or field of an input object
#[derive(Debug, Clone, Copy)]
pub enum InputType<'a> {
    Scalar(&'a ScalarType),
    Enum(&'a EnumType),
    InputObject(&'a InputObjectType),
}

impl InputType<'_> {
    pub fn name(&self) -> &str {
        match self {
            InputType::Scalar(scalar) => &scalar.name,
            InputType::Enum(en) => &en.name,
            InputType::InputObject(input_object) => &input_object.name,
        }
    }

    pub fn field(&self, name: &str) -> Option<&MetaInputValue> {
        match self {
            InputType::Scalar(_) | InputType::Enum(_) => None,
            InputType::InputObject(input_object) => input_object.input_fields.get(name),
        }
    }

    pub fn fields(&self) -> Box<dyn ExactSizeIterator<Item = &MetaInputValue> + '_> {
        match self {
            InputType::Scalar(_) | InputType::Enum(_) => Box::new(iter::empty()),
            InputType::InputObject(input_object) => Box::new(input_object.input_fields.values()),
        }
    }

    pub fn is_input_object(&self) -> bool {
        matches!(self, InputType::InputObject(_))
    }
}

impl<'a> TryFrom<&'a MetaType> for InputType<'a> {
    type Error = Error;

    fn try_from(value: &'a MetaType) -> Result<Self, Self::Error> {
        match value {
            MetaType::Scalar(scalar) => Ok(InputType::Scalar(scalar)),
            MetaType::Enum(en) => Ok(InputType::Enum(en)),
            MetaType::InputObject(object) => Ok(InputType::InputObject(object)),
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
    Object(&'a ObjectType),
    Interface(&'a InterfaceType),
    Union(&'a UnionType),
}

impl<'a> SelectionSetTarget<'a> {
    pub fn name(&self) -> &str {
        match self {
            SelectionSetTarget::Object(object) => &object.name,
            SelectionSetTarget::Interface(interface) => &interface.name,
            SelectionSetTarget::Union(union) => &union.name,
        }
    }

    pub fn field(&self, name: &str) -> Option<&'a MetaField> {
        if name == "__typename" {
            return Some(&*TYPENAME_FIELD);
        }

        match self {
            SelectionSetTarget::Object(obj) => obj.field_by_name(name),
            SelectionSetTarget::Interface(iface) => iface.fields.get(name),
            SelectionSetTarget::Union(_) => None,
        }
    }

    pub fn field_map(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            SelectionSetTarget::Object(obj) => Some(&obj.fields),
            SelectionSetTarget::Interface(iface) => Some(&iface.fields),
            SelectionSetTarget::Union(_) => None,
        }
    }
}

impl<'a> TryFrom<&'a MetaType> for SelectionSetTarget<'a> {
    type Error = Error;

    fn try_from(value: &'a MetaType) -> Result<Self, Self::Error> {
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

/// __typename is an annoying special case where the `MetaField` doesn't exist in schemas.
/// We need to fake it here instead...
static TYPENAME_FIELD: Lazy<MetaField> = Lazy::new(|| MetaField::new("__typename", "String!"));
