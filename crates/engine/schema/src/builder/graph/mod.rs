mod definitions;
mod directives;
mod selections;

use std::rc::Rc;

use builder::ValuePathSegment;
use extension_catalog::ExtensionId;
use fxhash::FxHashMap;
use introspection::IntrospectionSubgraph;
use rapidhash::RapidHashMap;
use selections::SelectionsBuilder;

use crate::*;

use super::{
    BuildContext, Error,
    interner::Interner,
    sdl::{self, SdlDefinition},
    value_path_to_string,
};

pub(crate) use definitions::*;
pub(crate) use directives::*;

pub(crate) struct GraphBuilder<'a> {
    pub ctx: BuildContext<'a>,
    pub definitions: Rc<GraphDefinitions<'a>>,
    pub graph: Graph,
    pub root_object_ids: Vec<ObjectDefinitionId>,
    pub required_scopes: Interner<RequiresScopesDirectiveRecord, RequiresScopesDirectiveId>,
    pub selections: SelectionsBuilder,

    // -- used for coercion
    pub value_path: Vec<ValuePathSegment>,
    pub input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, SchemaInputValueRecord)>>,
    pub virtual_subgraph_to_selection_set_resolver: Vec<Option<ExtensionId>>,
}

impl<'a> std::ops::Deref for GraphBuilder<'a> {
    type Target = BuildContext<'a>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl std::ops::DerefMut for GraphBuilder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

#[derive(Default)]
pub(crate) struct GraphDefinitions<'sdl> {
    pub type_name_to_id: RapidHashMap<&'sdl str, TypeDefinitionId>,
    pub site_id_to_sdl: FxHashMap<DirectiveSiteId, SdlDefinition<'sdl>>,
}

impl GraphDefinitions<'_> {
    fn get_type_id(&self, name: &str, span: sdl::Span) -> Result<TypeDefinitionId, Error> {
        let Some(id) = self.type_name_to_id.get(name) else {
            return Err((format!("Unknown type {name}"), span).into());
        };
        Ok(*id)
    }

    fn get_object_id(&self, name: &str, span: sdl::Span) -> Result<ObjectDefinitionId, Error> {
        let id = self.get_type_id(name, span)?;
        let TypeDefinitionId::Object(id) = id else {
            return Err((format!("Type {} is not an object", name), span).into());
        };
        Ok(id)
    }

    fn get_interface_id(&self, name: &str, span: sdl::Span) -> Result<InterfaceDefinitionId, Error> {
        let id = self.get_type_id(name, span)?;
        let TypeDefinitionId::Interface(id) = id else {
            return Err((format!("Type {} is not an interface", name), span).into());
        };
        Ok(id)
    }
}

impl GraphBuilder<'_> {
    pub(crate) fn value_path_string(&self) -> String {
        value_path_to_string(&self.ctx, &self.value_path)
    }

    fn parse_type(&self, ty: &str, span: sdl::Span) -> Result<TypeRecord, Error> {
        let mut wrappers = Vec::new();
        let mut chars = ty.chars().rev();

        let mut start = 0;
        let mut end = ty.len();
        loop {
            match chars.next() {
                Some('!') => {
                    wrappers.push(cynic_parser::common::WrappingType::NonNull);
                }
                Some(']') => {
                    wrappers.push(cynic_parser::common::WrappingType::List);
                    start += 1;
                }
                _ => break,
            }
            end -= 1;
        }
        Ok(TypeRecord {
            definition_id: self.definitions.get_type_id(&ty[start..end], span)?,
            wrapping: sdl::convert_wrappers(wrappers),
        })
    }

    pub(crate) fn definition_name_id(&self, ty: TypeDefinitionId) -> StringId {
        match ty {
            TypeDefinitionId::Scalar(id) => self.graph[id].name_id,
            TypeDefinitionId::Object(id) => self.graph[id].name_id,
            TypeDefinitionId::Interface(id) => self.graph[id].name_id,
            TypeDefinitionId::Union(id) => self.graph[id].name_id,
            TypeDefinitionId::Enum(id) => self.graph[id].name_id,
            TypeDefinitionId::InputObject(id) => self.graph[id].name_id,
        }
    }

    pub(crate) fn get_subgraph_id(&self, id: ResolverDefinitionId) -> SubgraphId {
        match &self.graph[id] {
            ResolverDefinitionRecord::FieldResolverExtension(record) => self.graph[record.directive_id].subgraph_id,
            ResolverDefinitionRecord::GraphqlFederationEntity(record) => record.endpoint_id.into(),
            ResolverDefinitionRecord::GraphqlRootField(record) => record.endpoint_id.into(),
            ResolverDefinitionRecord::Introspection => SubgraphId::Introspection,
            ResolverDefinitionRecord::Lookup(id) => self.get_subgraph_id(self.graph[*id].resolver_id),
            ResolverDefinitionRecord::Extension(record) => record.subgraph_id.into(),
            ResolverDefinitionRecord::SelectionSetResolverExtension(record) => record.subgraph_id.into(),
        }
    }

    pub(crate) fn type_name(&self, ty: TypeRecord) -> String {
        let name = &self.ctx[self.definition_name_id(ty.definition_id)];
        ty.wrapping.type_display(name).to_string()
    }
}
