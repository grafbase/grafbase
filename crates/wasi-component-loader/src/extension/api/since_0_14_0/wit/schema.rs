use wasmtime::component::{ComponentType, Lower};

use crate::state::WasiState;

impl Host for WasiState {}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct Schema {
    pub definitions: Vec<Definition>,
    pub directives: Vec<Directive>,
}

pub type DefinitionId = u32;

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(variant)]
pub enum Definition {
    #[component(name = "scalar")]
    Scalar(ScalarDefinition),
    #[component(name = "object")]
    Object(ObjectDefinition),
    #[component(name = "interface")]
    Interface(InterfaceDefinition),
    #[component(name = "union")]
    Union(UnionDefinition),
    #[component(name = "enum")]
    Enum(EnumDefinition),
    #[component(name = "input-object")]
    InputObject(InputObjectDefinition),
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct ScalarDefinition {
    pub id: DefinitionId,
    pub name: String,
    #[component(name = "specified-by-url")]
    pub specified_by_url: Option<String>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct ObjectDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub interfaces: Vec<DefinitionId>,
    pub fields: Vec<FieldDefinition>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct InterfaceDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub interfaces: Vec<DefinitionId>,
    pub fields: Vec<FieldDefinition>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct UnionDefinition {
    pub id: DefinitionId,
    pub name: String,
    #[component(name = "member-types")]
    pub member_types: Vec<DefinitionId>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct EnumDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub values: Vec<EnumValue>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct InputObjectDefinition {
    pub id: DefinitionId,
    pub name: String,
    #[component(name = "input-fields")]
    pub input_fields: Vec<InputValueDefinition>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct FieldDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub ty: Ty,
    pub arguments: Vec<InputValueDefinition>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct Ty {
    pub wrapping: Vec<WrappingType>,
    #[component(name = "definition-id")]
    pub definition_id: DefinitionId,
}

#[derive(Clone, Copy, Debug, ComponentType, Lower)]
#[component(enum)]
#[repr(u8)]
pub enum WrappingType {
    #[component(name = "non-null")]
    NonNull,
    #[component(name = "list")]
    List,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct InputValueDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub ty: Ty,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct EnumValue {
    pub name: String,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct Directive {
    pub name: String,
    pub arguments: Vec<u8>,
}

pub trait Host: Send + ::core::marker::Send {}
pub trait GetHost<T, D>: Fn(T) -> <Self as GetHost<T, D>>::Host + Send + Sync + Copy + 'static {
    type Host: Host + Send;
}
impl<F, T, D, O> GetHost<T, D> for F
where
    F: Fn(T) -> O + Send + Sync + Copy + 'static,
    O: Host + Send,
{
    type Host = O;
}
pub fn add_to_linker_get_host<T, G: for<'a> GetHost<&'a mut T, T, Host: Host + Send>>(
    _linker: &mut wasmtime::component::Linker<T>,
    _host_getter: G,
) -> wasmtime::Result<()>
where
    T: Send,
{
    Ok(())
}
pub fn add_to_linker<T, U>(
    linker: &mut wasmtime::component::Linker<T>,
    get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
) -> wasmtime::Result<()>
where
    U: Host + Send,
    T: Send,
{
    add_to_linker_get_host(linker, get)
}
impl<_T: Host + ?Sized + Send> Host for &mut _T {}

