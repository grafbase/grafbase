use wasmtime::component::{ComponentType, Lower};

pub use crate::extension::api::since_0_8_0::wit::directive::{
    self as since_0_8_0, DirectiveSite, EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite,
    GetHost, Host, InterfaceDirectiveSite, ObjectDirectiveSite, ScalarDirectiveSite, SchemaDirective,
    UnionDirectiveSite, add_to_linker, add_to_linker_get_host,
};

#[derive(Debug, ComponentType, Lower)]
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
    pub directive_names: Vec<(&'a str, u32, u32)>,
    pub elements: Vec<QueryElement<'a>>,
}

impl<'a> From<QueryElements<'a>> for since_0_8_0::QueryElements<'a> {
    fn from(value: QueryElements<'a>) -> Self {
        Self {
            directive_names: value.directive_names,
            elements: value.elements.into_iter().map(Into::into).collect(),
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
