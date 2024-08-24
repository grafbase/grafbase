use itertools::Itertools;

use crate::{
    bindings::{
        component::grafbase::types::Error,
        exports::component::grafbase::authorization::{self, EdgeDefinition, SharedContext},
    },
    contract_error, error, init_logging, maybe_read_input, read_input, Component, Metadata, RUNTIME,
};

mod service;

impl authorization::Guest for Component {
    fn authorize_edge_pre_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        arguments: String,
        _metadata: String,
    ) -> Result<(), Error> {
        init_logging();

        match (definition.parent_type_name.as_str(), definition.field_name.as_str()) {
            ("Query", "user") => {
                tracing::info!("Authorizing access to Query.user with {arguments}",);

                #[derive(serde::Deserialize)]
                struct Arguments {
                    id: usize,
                }
                let arguments: Arguments = read_input(&arguments)?;

                if context.get("current-user-id").and_then(|id| id.parse().ok()) == Some(arguments.id) {
                    Ok(())
                } else {
                    Err(error("Unauthorized"))
                }
            }
            _ => Err(contract_error()),
        }
    }

    fn authorize_parent_edge_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        parents: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        init_logging();

        match (definition.parent_type_name.as_str(), definition.field_name.as_str()) {
            ("User", "address") => {
                tracing::info!("Authorizing access to User.address for: {}", parents.iter().join(", "));

                #[derive(Debug, serde::Deserialize)]
                struct User {
                    id: usize,
                }

                let metadata: Metadata = maybe_read_input(&metadata);
                if let Some(role) = metadata.allow_role {
                    if context.get("role") == Some(role.clone()) {
                        tracing::info!("Granting access to role {role}");
                        return (0..parents.len()).map(|_| Ok(())).collect();
                    }
                }

                let parents: Vec<User> = parents
                    .into_iter()
                    .map(|parent| read_input(&parent))
                    .collect::<Result<_, _>>()
                    .unwrap();

                let Some(current_user_id) = context.get("current-user-id").and_then(|id| id.parse().ok()) else {
                    return (0..parents.len()).map(|_| Err(error("No current user id"))).collect();
                };

                RUNTIME.block_on(service::authorize_address(
                    current_user_id,
                    parents.into_iter().map(|User { id, .. }| id).collect(),
                ))
            }
            _ => vec![Err(contract_error())],
        }
    }

    fn authorize_edge_node_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        nodes: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        init_logging();

        match (definition.parent_type_name.as_str(), definition.field_name.as_str()) {
            ("Query", "users") => {
                tracing::info!("Authorizing access to Query.users for: {}", nodes.iter().join(", "));

                #[derive(Debug, serde::Deserialize)]
                struct User {
                    id: usize,
                }

                let metadata: Metadata = maybe_read_input(&metadata);
                if let Some(role) = metadata.allow_role {
                    if context.get("role") == Some(role.clone()) {
                        tracing::info!("Granting access to role {role}");
                        return (0..nodes.len()).map(|_| Ok(())).collect();
                    }
                }

                let nodes: Vec<User> = nodes
                    .into_iter()
                    .map(|node| read_input(&node))
                    .collect::<Result<_, _>>()
                    .unwrap();

                let Some(current_user_id) = context.get("current-user-id").and_then(|id| id.parse().ok()) else {
                    return (0..nodes.len()).map(|_| Err(error("No current user id"))).collect();
                };

                RUNTIME.block_on(service::authorize_user(
                    current_user_id,
                    nodes.into_iter().map(|User { id, .. }| id).collect(),
                ))
            }
            _ => vec![Err(contract_error())],
        }
    }

    fn authorize_node_pre_execution(
        _context: SharedContext,
        _definition: authorization::NodeDefinition,
        _metadata: String,
    ) -> Result<(), Error> {
        init_logging();

        Err(error("Not implemented"))
    }
}
