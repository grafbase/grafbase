use engine_value::ConstValue;
use indexmap::{IndexMap, IndexSet};
use registry_v2::{CacheControl, ScalarParser, UnionDiscriminator};

use crate::{constraint::Constraint, fields::MetaField, EnumType, MetaInputValue};

#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub enum MetaType {
    Scalar(ScalarType),
    Object(ObjectType),
    Interface(InterfaceType),
    Union(UnionType),
    Enum(EnumType),
    InputObject(InputObjectType),
}

impl MetaType {
    pub fn object(&self) -> Option<&ObjectType> {
        match self {
            MetaType::Object(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, MetaType::Object(_))
    }

    pub fn is_input_object(&self) -> bool {
        matches!(self, MetaType::InputObject(_))
    }

    pub fn is_node(&self) -> bool {
        match self {
            MetaType::Object(object) => object.is_node,
            _ => false,
        }
    }

    pub fn get_input_field(&self, name: &str) -> Option<&MetaInputValue> {
        if let MetaType::InputObject(ref object) = self {
            object.input_fields.get(name)
        } else {
            None
        }
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct ScalarType {
    pub name: String,
    pub description: Option<String>,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub is_valid: Option<fn(value: &ConstValue) -> bool>,
    pub specified_by_url: Option<String>,
    #[serde(default)]
    pub parser: ScalarParser,
}

impl From<ScalarType> for MetaType {
    fn from(val: ScalarType) -> Self {
        MetaType::Scalar(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectType {
    pub name: String,
    pub description: Option<String>,
    pub fields: IndexMap<String, MetaField>,
    pub cache_control: Option<Box<CacheControl>>,
    pub extends: bool,
    pub is_subscription: bool,
    /// Define if the current Object if a Node
    pub is_node: bool,
    pub rust_typename: String,
    pub constraints: Vec<Constraint>,
    pub external: bool,
    pub shareable: bool,
}

impl ObjectType {
    pub fn new(name: impl Into<String>, fields: impl IntoIterator<Item = MetaField>) -> ObjectType {
        let name = name.into();
        ObjectType {
            rust_typename: name.clone(),
            name,
            fields: fields.into_iter().map(|field| (field.name.clone(), field)).collect(),
            description: None,
            cache_control: Default::default(),
            extends: false,
            is_subscription: false,
            is_node: false,
            constraints: vec![],
            external: false,
            shareable: false,
        }
    }

    pub fn with_description(self, description: impl Into<Option<String>>) -> Self {
        ObjectType {
            description: description.into(),
            ..self
        }
    }

    pub fn with_cache_control(self, cache_control: Option<Box<CacheControl>>) -> Self {
        ObjectType { cache_control, ..self }
    }

    pub fn with_external(self, external: bool) -> Self {
        ObjectType { external, ..self }
    }

    pub fn with_shareable(self, shareable: bool) -> Self {
        ObjectType { shareable, ..self }
    }

    #[inline]
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields.get(name)
    }
}

impl From<ObjectType> for MetaType {
    fn from(val: ObjectType) -> Self {
        MetaType::Object(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterfaceType {
    pub name: String,
    pub description: Option<String>,
    pub fields: IndexMap<String, MetaField>,
    pub cache_control: Option<Box<CacheControl>>,
    pub possible_types: IndexSet<String>,
    pub extends: bool,
    pub rust_typename: String,
}

impl InterfaceType {
    pub fn new(name: impl Into<String>, fields: impl IntoIterator<Item = MetaField>) -> Self {
        InterfaceType {
            name: name.into(),
            description: None,
            fields: fields.into_iter().map(|field| (field.name.clone(), field)).collect(),
            possible_types: Default::default(),
            cache_control: Default::default(),
            extends: false,
            rust_typename: String::new(),
        }
    }

    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields.get(name)
    }

    pub fn with_description(self, description: impl Into<Option<String>>) -> Self {
        InterfaceType {
            description: description.into(),
            ..self
        }
    }

    pub fn with_cache_control(self, cache_control: Option<Box<CacheControl>>) -> Self {
        InterfaceType { cache_control, ..self }
    }
}

impl From<InterfaceType> for MetaType {
    fn from(val: InterfaceType) -> Self {
        MetaType::Interface(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct UnionType {
    pub name: String,
    pub description: Option<String>,
    pub possible_types: IndexSet<String>,
    pub rust_typename: String,
    pub discriminators: Option<Vec<(String, UnionDiscriminator)>>,
}

impl UnionType {
    pub fn new<T: Into<String>>(name: impl Into<String>, possible_types: impl IntoIterator<Item = T>) -> UnionType {
        let name = name.into();
        UnionType {
            rust_typename: name.clone(),
            name,
            description: None,
            possible_types: possible_types.into_iter().map(Into::into).collect(),
            discriminators: None,
        }
    }
}

impl From<UnionType> for MetaType {
    fn from(val: UnionType) -> Self {
        MetaType::Union(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct InputObjectType {
    pub name: String,
    pub description: Option<String>,
    pub input_fields: IndexMap<String, MetaInputValue>,
    pub rust_typename: String,
    pub oneof: bool,
}

impl InputObjectType {
    pub fn new(name: String, input_fields: impl IntoIterator<Item = MetaInputValue>) -> Self {
        InputObjectType {
            rust_typename: name.clone(),
            name,
            description: None,
            input_fields: input_fields.into_iter().map(|v| (v.name.clone(), v)).collect(),
            oneof: false,
        }
    }

    pub fn with_description(self, description: Option<String>) -> Self {
        InputObjectType { description, ..self }
    }

    pub fn with_oneof(self, oneof: bool) -> Self {
        InputObjectType { oneof, ..self }
    }
}

impl From<InputObjectType> for MetaType {
    fn from(val: InputObjectType) -> Self {
        MetaType::InputObject(val)
    }
}

impl MetaType {
    #[inline]
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields().and_then(|fields| fields.get(name))
    }

    #[inline]
    pub fn field_by_name_mut(&mut self, name: &str) -> Option<&mut MetaField> {
        self.fields_mut().and_then(|fields| fields.get_mut(name))
    }

    #[inline]
    pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            MetaType::Object(inner) => Some(&inner.fields),
            MetaType::Interface(inner) => Some(&inner.fields),
            _ => None,
        }
    }

    #[inline]
    pub fn fields_mut(&mut self) -> Option<&mut IndexMap<String, MetaField>> {
        match self {
            MetaType::Object(inner) => Some(&mut inner.fields),
            MetaType::Interface(inner) => Some(&mut inner.fields),
            _ => None,
        }
    }

    pub fn constraints(&self) -> &[Constraint] {
        match self {
            MetaType::Object(inner) => &inner.constraints,
            _ => &[],
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        match self {
            MetaType::Scalar(inner) => &inner.name,
            MetaType::Object(inner) => &inner.name,
            MetaType::Interface(inner) => &inner.name,
            MetaType::Union(inner) => &inner.name,
            MetaType::Enum(inner) => &inner.name,
            MetaType::InputObject(inner) => &inner.name,
        }
    }

    #[inline]
    pub fn description(&self) -> Option<&str> {
        match self {
            MetaType::Scalar(inner) => inner.description.as_deref(),
            MetaType::Object(inner) => inner.description.as_deref(),
            MetaType::Interface(inner) => inner.description.as_deref(),
            MetaType::Union(inner) => inner.description.as_deref(),
            MetaType::Enum(inner) => inner.description.as_deref(),
            MetaType::InputObject(inner) => inner.description.as_deref(),
        }
    }

    #[inline]
    pub fn is_composite(&self) -> bool {
        matches!(self, MetaType::Object(_) | MetaType::Interface(_) | MetaType::Union(_))
    }

    #[inline]
    pub fn is_abstract(&self) -> bool {
        matches!(self, MetaType::Interface(_) | MetaType::Union(_))
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(self, MetaType::Enum(_) | MetaType::Scalar(_))
    }

    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(self, MetaType::Enum(_) | MetaType::Scalar(_) | MetaType::InputObject(_))
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        matches!(self, MetaType::Enum(_))
    }

    #[inline]
    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            MetaType::Interface(inner) => inner.possible_types.contains(type_name),
            MetaType::Union(inner) => inner.possible_types.contains(type_name),
            MetaType::Object(inner) => inner.name == type_name,
            _ => false,
        }
    }

    #[inline]
    pub fn possible_types(&self) -> Option<&IndexSet<String>> {
        match self {
            MetaType::Interface(inner) => Some(&inner.possible_types),
            MetaType::Union(inner) => Some(&inner.possible_types),
            _ => None,
        }
    }

    pub fn type_overlap(&self, ty: &MetaType) -> bool {
        if std::ptr::eq(self, ty) {
            return true;
        }

        match (self.is_abstract(), ty.is_abstract()) {
            (true, true) => self
                .possible_types()
                .iter()
                .copied()
                .flatten()
                .any(|type_name| ty.is_possible_type(type_name)),
            (true, false) => self.is_possible_type(ty.name()),
            (false, true) => ty.is_possible_type(self.name()),
            (false, false) => false,
        }
    }

    pub fn rust_typename(&self) -> Option<&String> {
        match self {
            MetaType::Scalar { .. } => None,
            MetaType::Object(inner) => Some(&inner.rust_typename),
            MetaType::Interface(inner) => Some(&inner.rust_typename),
            MetaType::Union(inner) => Some(&inner.rust_typename),
            MetaType::Enum(inner) => Some(&inner.rust_typename),
            MetaType::InputObject(inner) => Some(&inner.rust_typename),
        }
    }
}
