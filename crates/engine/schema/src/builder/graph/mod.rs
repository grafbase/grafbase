mod definitions;
mod field_set;
mod input_values;
mod post_process;

use std::collections::BTreeMap;

use builder::{SchemaLocation, ValuePathSegment, extension::finalize_selection_set_resolvers};
use extension_catalog::ExtensionId;
use fxhash::FxHashMap;
use introspection::IntrospectionMetadata;
use post_process::process_directives;

use crate::*;

use super::{BuildError, Context, interner::Interner};

pub(crate) struct GraphContext<'a> {
    pub ctx: Context<'a>,
    pub graph: Graph,
    pub required_scopes: Interner<RequiresScopesDirectiveRecord, RequiresScopesDirectiveId>,
    pub scalar_mapping: FxHashMap<federated_graph::ScalarDefinitionId, ScalarDefinitionId>,
    pub enum_mapping: FxHashMap<federated_graph::EnumDefinitionId, EnumDefinitionId>,
    pub input_value_mapping: FxHashMap<federated_graph::InputValueDefinitionId, InputValueDefinitionId>,
    pub graphql_federated_entity_resolvers: FxHashMap<(EntityDefinitionId, GraphqlEndpointId), Vec<EntityResovler>>,
    // -- used for field sets
    pub deduplicated_fields: BTreeMap<SchemaFieldRecord, SchemaFieldId>,
    pub field_arguments: Vec<SchemaFieldArgumentRecord>,
    // -- used for coercion
    pub value_path: Vec<ValuePathSegment>,
    pub input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, SchemaInputValueRecord)>>,
    pub virtual_subgraph_to_selection_set_resolver: Vec<Option<ExtensionId>>,
}

impl<'a> std::ops::Deref for GraphContext<'a> {
    type Target = Context<'a>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl std::ops::DerefMut for GraphContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

#[derive(Clone)]
pub(crate) enum EntityResovler {
    Root(ResolverDefinitionId),
    Entity {
        key: federated_graph::SelectionSet,
        id: ResolverDefinitionId,
    },
}

impl EntityResovler {
    fn id(&self) -> ResolverDefinitionId {
        match self {
            EntityResovler::Root(id) | EntityResovler::Entity { id, .. } => *id,
        }
    }
}

impl Context<'_> {
    pub(crate) fn into_ctx_graph_introspection(self) -> Result<(Self, Graph, IntrospectionMetadata), BuildError> {
        let (mut ctx, locations, introspection) = self.into_graph_context()?;

        // From this point on the definitions should have been all added and now we interpret the
        // directives.

        for (ix, extension) in ctx.federated_graph.extensions.iter().enumerate() {
            let extension_id = federated_graph::ExtensionId::from(ix);
            for directive in &extension.schema_directives {
                let id = ctx.ingest_extension_directive(
                    SchemaLocation::SchemaDirective(directive.subgraph_id),
                    directive.subgraph_id,
                    extension_id,
                    directive.name,
                    &directive.arguments,
                )?;
                ctx.push_extension_schema_directive(id);
            }
        }

        process_directives(&mut ctx, locations)?;

        finalize_selection_set_resolvers(&mut ctx)?;

        let GraphContext {
            ctx,
            mut graph,
            required_scopes,
            deduplicated_fields,
            field_arguments,
            ..
        } = ctx;
        graph.required_scopes = required_scopes.into();
        let mut fields = deduplicated_fields.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|(_, id)| *id);
        graph.fields = fields.into_iter().map(|(field, _)| field).collect();
        graph.field_arguments = field_arguments;

        Ok((ctx, graph, introspection))
    }
}

impl GraphContext<'_> {
    fn convert_type(&self, federated_graph::Type { wrapping, definition }: federated_graph::Type) -> TypeRecord {
        TypeRecord {
            definition_id: self.convert_definition(definition),
            wrapping,
        }
    }

    fn convert_definition(&self, definition: federated_graph::Definition) -> TypeDefinitionId {
        match definition {
            federated_graph::Definition::Scalar(id) => TypeDefinitionId::Scalar(self.scalar_mapping[&id]),
            federated_graph::Definition::Object(id) => TypeDefinitionId::Object(id.into()),
            federated_graph::Definition::Interface(id) => TypeDefinitionId::Interface(id.into()),
            federated_graph::Definition::Union(id) => TypeDefinitionId::Union(id.into()),
            federated_graph::Definition::Enum(id) => TypeDefinitionId::Enum(self.enum_mapping[&id]),
            federated_graph::Definition::InputObject(id) => TypeDefinitionId::InputObject(id.into()),
        }
    }

    pub(crate) fn type_name(&self, ty: TypeRecord) -> String {
        let name = match ty.definition_id {
            TypeDefinitionId::Scalar(id) => &self.ctx.strings[self.graph[id].name_id],
            TypeDefinitionId::Object(id) => &self.ctx.strings[self.graph[id].name_id],
            TypeDefinitionId::Interface(id) => &self.ctx.strings[self.graph[id].name_id],
            TypeDefinitionId::Union(id) => &self.ctx.strings[self.graph[id].name_id],
            TypeDefinitionId::Enum(id) => &self.ctx.strings[self.graph[id].name_id],
            TypeDefinitionId::InputObject(id) => &self.ctx.strings[self.graph[id].name_id],
        };
        let mut s = String::new();
        ty.wrapping.write_type_string(name, &mut s).unwrap();
        s
    }
}

macro_rules! from_id_newtypes {
    ($($from:ty => $name:ident,)*) => {
        $(
            impl From<$from> for $name {
                fn from(id: $from) -> Self {
                    $name::from(usize::from(id))
                }
            }
        )*
    }
}

// EnumValueId from federated_graph can't be directly
// converted, we sort them by their name.
from_id_newtypes! {
    federated_graph::InputObjectId => InputObjectDefinitionId,
    federated_graph::InterfaceId => InterfaceDefinitionId,
    federated_graph::ObjectId => ObjectDefinitionId,
    federated_graph::UnionId => UnionDefinitionId,
    federated_graph::FieldId => FieldDefinitionId,
}

impl From<federated_graph::EntityDefinitionId> for EntityDefinitionId {
    fn from(id: federated_graph::EntityDefinitionId) -> Self {
        match id {
            federated_graph::EntityDefinitionId::Object(id) => EntityDefinitionId::Object(id.into()),
            federated_graph::EntityDefinitionId::Interface(id) => EntityDefinitionId::Interface(id.into()),
        }
    }
}
