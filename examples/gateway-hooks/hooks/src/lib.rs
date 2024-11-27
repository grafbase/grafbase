mod common;
use common::*;
use grafbase_hooks::{grafbase_hooks, Context, EdgeDefinition, Error, ErrorResponse, Headers, Hooks, SharedContext};
use itertools::Itertools;

// Individual interface implementations
mod authorization;

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, context: Context, headers: Headers) -> Result<(), ErrorResponse> {
        init_logging();

        if let Some(id) = headers.get("x-current-user-id") {
            tracing::info!("Current user: {id}");
            context.set("current-user-id", &id);
        }

        if let Some(role) = headers.get("x-role") {
            tracing::info!("Current role: {role}");
            context.set("role", &role);
        }

        Ok(())
    }

    fn authorize_edge_pre_execution(
        &mut self,
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
        &mut self,
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

                authorization::authorize_address(
                    current_user_id,
                    parents.into_iter().map(|User { id, .. }| id).collect(),
                )
            }
            _ => vec![Err(contract_error())],
        }
    }

    fn authorize_edge_node_post_execution(
        &mut self,
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

                authorization::authorize_user(current_user_id, nodes.into_iter().map(|User { id, .. }| id).collect())
            }
            _ => vec![Err(contract_error())],
        }
    }
}

grafbase_hooks::register_hooks!(Component);
