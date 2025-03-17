use wasmtime::component::{ComponentType, Lower};

pub use crate::extension::api::since_0_8_0::wit::directive::{
    DirectiveSite, EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite, Host,
    InterfaceDirectiveSite, ObjectDirectiveSite, ScalarDirectiveSite, SchemaDirective, UnionDirectiveSite,
    add_to_linker,
};
use crate::{extension::api::since_0_8_0::wit::directive as since_0_8_0, state::WasiState};

impl Host for WasiState {}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct QueryElement<'a> {
    pub id: u32,
    pub site: DirectiveSite<'a>,
    pub arguments: Vec<u8>,
}

impl<'a> From<QueryElement<'a>> for since_0_8_0::QueryElement<'a> {
    fn from(value: QueryElement<'a>) -> Self {
        Self {
            site: value.site,
            arguments: value.arguments,
        }
    }
}

#[derive(Debug, ComponentType, Lower)]
#[component(record)]
pub struct QueryElements<'a> {
    #[component(name = "directive-names")]
    pub directive_names: &'a [(&'a str, u32, u32)],
    pub elements: &'a [QueryElement<'a>],
}

impl<'a> From<QueryElements<'a>> for since_0_8_0::QueryElements<'a> {
    fn from(value: QueryElements<'a>) -> Self {
        Self {
            directive_names: value.directive_names.to_vec(),
            elements: value.elements.iter().map(|e| e.clone().into()).collect(),
        }
    }
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
