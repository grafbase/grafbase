use std::{collections::HashMap, sync::Arc};

use crate::gateway::DynHookContext;
use engine_schema::{Subgraph, SubgraphId};
use extension_catalog::{ExtensionId, Id};
use runtime::extension::Data;
use serde::Serialize;
use tokio::sync::Mutex;

use super::{
    AuthenticationTestExtension, AuthorizationTestExtension, FieldResolverTestExtension,
    FieldResolverTestExtensionBuilder, SelectionSetResolverTestExtension, SelectionSetResolverTestExtensionBuilder,
};

#[derive(Default)]
pub struct TestExtensionsState {
    pub authentication: HashMap<ExtensionId, Arc<dyn AuthenticationTestExtension>>,
    pub authorization: HashMap<ExtensionId, Arc<dyn AuthorizationTestExtension>>,
    pub selection_set_resolver_builders: HashMap<ExtensionId, Arc<dyn SelectionSetResolverTestExtensionBuilder>>,
    pub selection_set_resolvers: HashMap<(ExtensionId, SubgraphId), Arc<dyn SelectionSetResolverTestExtension>>,
    pub field_resolver_builders: HashMap<ExtensionId, Arc<dyn FieldResolverTestExtensionBuilder>>,
    pub field_resolvers: HashMap<(ExtensionId, SubgraphId), Arc<dyn FieldResolverTestExtension>>,
}

impl TestExtensionsState {
    pub(super) fn get_field_resolver_ext(
        &mut self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'_>,
    ) -> Arc<dyn FieldResolverTestExtension> {
        self.field_resolvers
            .entry((extension_id, subgraph.id()))
            .or_insert_with(|| {
                self.field_resolver_builders.get(&extension_id).unwrap().build(
                    subgraph
                        .extension_schema_directives()
                        .filter(|dir| dir.extension_id == extension_id)
                        .map(|dir| (dir.name(), serde_json::to_value(dir.static_arguments()).unwrap()))
                        .collect(),
                )
            })
            .clone()
    }

    pub(super) fn get_selection_set_resolver_ext(
        &mut self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'_>,
    ) -> Arc<dyn SelectionSetResolverTestExtension> {
        self.selection_set_resolvers
            .entry((extension_id, subgraph.id()))
            .or_insert_with(|| {
                self.selection_set_resolver_builders.get(&extension_id).unwrap().build(
                    subgraph
                        .extension_schema_directives()
                        .filter(|dir| dir.extension_id == extension_id)
                        .map(|dir| (dir.name(), serde_json::to_value(dir.static_arguments()).unwrap()))
                        .collect(),
                )
            })
            .clone()
    }

    pub(super) fn get_authentication_ext(&self, extension_id: ExtensionId) -> Arc<dyn AuthenticationTestExtension> {
        Arc::clone(self.authentication.get(&extension_id).unwrap())
    }

    pub(super) fn get_authorization_ext(&self, extension_id: ExtensionId) -> Arc<dyn AuthorizationTestExtension> {
        Arc::clone(self.authorization.get(&extension_id).unwrap())
    }
}

pub struct TestManifest {
    pub id: Id,
    pub sdl: Option<&'static str>,
    pub r#type: extension_catalog::Type,
}

#[derive(Default, Clone)]
pub struct TestExtensions {
    pub(super) state: Arc<Mutex<TestExtensionsState>>,
}

impl runtime::extension::ExtensionRuntime for TestExtensions {
    type Context = DynHookContext;
}

pub fn json_data(value: impl Serialize) -> Data {
    Data::Json(serde_json::to_vec(&value).unwrap().into())
}
