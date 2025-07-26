use wasmtime::component::{ComponentType, Lower};

use crate::state::InstanceState;

impl Host for InstanceState {}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct QueryElement<'a> {
    pub id: u32,
    pub site: DirectiveSite<'a>,
    pub arguments: Vec<u8>,
}

#[derive(Debug, ComponentType, Lower)]
#[component(record)]
pub struct QueryElements<'a> {
    #[component(name = "directive-names")]
    pub directive_names: &'a [(&'a str, u32, u32)],
    pub elements: &'a [QueryElement<'a>],
}

#[derive(Debug, ComponentType, Lower)]
#[component(record)]
pub struct ResponseElements<'a> {
    #[component(name = "directive-names")]
    pub directive_names: Vec<(&'a str, u32, u32)>,
    pub elements: Vec<ResponseElement>,
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug, ComponentType, Lower)]
#[component(record)]
pub struct ResponseElement {
    #[component(name = "query-element-id")]
    pub query_element_id: u32,
    #[component(name = "items-range")]
    pub items_range: (u32, u32),
}

#[derive(Debug, ComponentType, Lower)]
#[component(record)]
pub struct SchemaDirective<'a> {
    #[component(name = "subgraph-name")]
    pub subgraph_name: &'a str,
    pub name: &'a str,
    pub arguments: Vec<u8>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct ObjectDirectiveSite<'a> {
    #[component(name = "object-name")]
    pub object_name: &'a str,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct FieldDefinitionDirectiveSite<'a> {
    #[component(name = "parent-type-name")]
    pub parent_type_name: &'a str,
    #[component(name = "field-name")]
    pub field_name: &'a str,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct FieldDefinitionDirective<'a> {
    pub name: &'a str,
    pub site: FieldDefinitionDirectiveSite<'a>,
    pub arguments: &'a [u8],
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct UnionDirectiveSite<'a> {
    #[component(name = "union-name")]
    pub union_name: &'a str,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct InterfaceDirectiveSite<'a> {
    #[component(name = "interface-name")]
    pub interface_name: &'a str,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct EnumDirectiveSite<'a> {
    #[component(name = "enum-name")]
    pub enum_name: &'a str,
}
#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct ScalarDirectiveSite<'a> {
    #[component(name = "scalar-name")]
    pub scalar_name: &'a str,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(variant)]
pub enum DirectiveSite<'a> {
    #[component(name = "scalar")]
    Scalar(ScalarDirectiveSite<'a>),
    #[component(name = "object")]
    Object(ObjectDirectiveSite<'a>),
    #[component(name = "field-definition")]
    FieldDefinition(FieldDefinitionDirectiveSite<'a>),
    #[component(name = "interface")]
    Interface(InterfaceDirectiveSite<'a>),
    #[component(name = "union")]
    Union(UnionDirectiveSite<'a>),
    #[component(name = "enum")]
    Enum(EnumDirectiveSite<'a>),
}

// Typical Wasmtime bindgen! macro generated stuff
pub trait Host: Send + ::core::marker::Send {}
impl<_T: Host + ?Sized + Send> Host for &mut _T {}
pub fn add_to_linker<T, D>(
    _linker: &mut wasmtime::component::Linker<T>,
    _host_getter: fn(&mut T) -> D::Data<'_>,
) -> wasmtime::Result<()>
where
    D: wasmtime::component::HasData,
    for<'a> D::Data<'a>: Host,
    T: 'static + Send,
{
    Ok(())
}

impl<'a> From<engine_schema::DirectiveSite<'a>> for DirectiveSite<'a> {
    fn from(value: engine_schema::DirectiveSite<'a>) -> Self {
        match value {
            engine_schema::DirectiveSite::Scalar(def) => Self::Scalar(ScalarDirectiveSite {
                scalar_name: def.name(),
            }),
            engine_schema::DirectiveSite::Object(def) => Self::Object(ObjectDirectiveSite {
                object_name: def.name(),
            }),
            engine_schema::DirectiveSite::Field(def) => Self::FieldDefinition(FieldDefinitionDirectiveSite {
                parent_type_name: def.parent_entity().name(),
                field_name: def.name(),
            }),
            engine_schema::DirectiveSite::Interface(def) => Self::Interface(InterfaceDirectiveSite {
                interface_name: def.name(),
            }),
            engine_schema::DirectiveSite::Union(def) => Self::Union(UnionDirectiveSite { union_name: def.name() }),
            engine_schema::DirectiveSite::Enum(def) => Self::Enum(EnumDirectiveSite { enum_name: def.name() }),
            engine_schema::DirectiveSite::InputObject(_)
            | engine_schema::DirectiveSite::EnumValue(_)
            | engine_schema::DirectiveSite::InputValue(_) => unimplemented!("Not used"),
        }
    }
}
