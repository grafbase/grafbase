mod definitions;
mod directives;

use std::collections::BTreeMap;

use builder::ValuePathSegment;
use extension_catalog::ExtensionId;
use fxhash::FxHashMap;
use introspection::IntrospectionSubgraph;
use rapidhash::RapidHashMap;

use crate::*;

use super::{BuildContext, Error, interner::Interner, sdl, value_path_to_string};

pub(crate) use definitions::*;
pub(crate) use directives::*;

pub(crate) struct GraphBuilder<'a> {
    pub ctx: BuildContext<'a>,
    pub graph: Graph,
    pub required_scopes: Interner<RequiresScopesDirectiveRecord, RequiresScopesDirectiveId>,
    pub type_definitions: RapidHashMap<&'a str, TypeDefinitionId>,
    pub entity_resolvers: FxHashMap<(EntityDefinitionId, SubgraphId), Vec<ResolverDefinitionId>>,
    pub composite_entity_keys: FxHashMap<(EntityDefinitionId, SubgraphId), Vec<FieldSetRecord>>,

    // -- used for field sets
    pub deduplicated_fields: BTreeMap<SchemaFieldRecord, SchemaFieldId>,
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

impl GraphBuilder<'_> {
    pub(crate) fn value_path_string(&self) -> String {
        value_path_to_string(&self.ctx, &self.value_path)
    }

    fn get_type_id(&self, name: &str, span: sdl::Span) -> Result<TypeDefinitionId, Error> {
        let Some(id) = self.type_definitions.get(name) else {
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
            definition_id: self.get_type_id(&ty[start..end], span)?,
            wrapping: sdl::convert_wrappers(wrappers),
        })
    }

    pub(crate) fn type_name(&self, ty: TypeRecord) -> String {
        let name = match ty.definition_id {
            TypeDefinitionId::Scalar(id) => &self.ctx[self.graph[id].name_id],
            TypeDefinitionId::Object(id) => &self.ctx[self.graph[id].name_id],
            TypeDefinitionId::Interface(id) => &self.ctx[self.graph[id].name_id],
            TypeDefinitionId::Union(id) => &self.ctx[self.graph[id].name_id],
            TypeDefinitionId::Enum(id) => &self.ctx[self.graph[id].name_id],
            TypeDefinitionId::InputObject(id) => &self.ctx[self.graph[id].name_id],
        };
        let mut s = String::new();
        ty.wrapping.write_type_string(name, &mut s).unwrap();
        s
    }
}
