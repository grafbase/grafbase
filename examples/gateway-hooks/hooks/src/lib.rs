mod common;
use common::*;
use grafbase_hooks::{
    Context, EdgeNodePostExecutionArguments, EdgePreExecutionArguments, Error, ErrorResponse, Headers, Hooks,
    ParentEdgePostExecutionArguments, SharedContext, grafbase_hooks,
};

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

    fn on_gateway_request(&mut self, context: Context, url: String, headers: Headers) -> Result<(), ErrorResponse> {
        init_logging();

        if let Some(id) = headers.get("x-current-user-id") {
            tracing::info!("Current user: {id}");
            context.set("current-user-id", &id);
        }

        if let Some(role) = headers.get("x-role") {
            tracing::info!("Current role: {role}");
            context.set("role", &role);
        }

        context.set("url", &url);

        Ok(())
    }

    fn authorize_edge_pre_execution(
        &mut self,
        context: SharedContext,
        arguments: EdgePreExecutionArguments,
    ) -> Result<(), Error> {
        init_logging();

        match (arguments.parent_type_name(), arguments.field_name()) {
            ("Query", "user") => {
                tracing::info!("Authorizing access to Query.user",);

                #[derive(serde::Deserialize)]
                struct Arguments {
                    id: usize,
                }

                let arguments: Arguments = arguments.arguments().map_err(|err| {
                    tracing::error!("Failed to deserialize input: {err}");
                    contract_error()
                })?;

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
        arguments: ParentEdgePostExecutionArguments,
    ) -> Vec<Result<(), Error>> {
        init_logging();

        match (arguments.parent_type_name(), arguments.field_name()) {
            ("User", "address") => {
                tracing::info!("Authorizing access to User.address");

                #[derive(Debug, serde::Deserialize)]
                struct User {
                    id: usize,
                }

                let metadata: Metadata = arguments.metadata().unwrap_or_default();

                let parents: Vec<User> = match arguments.parents() {
                    Ok(parents) => parents,
                    Err(_) => return vec![Err(contract_error())],
                };

                if let Some(role) = metadata.allow_role {
                    if context.get("role") == Some(role.clone()) {
                        tracing::info!("Granting access to role {role}");
                        return (0..parents.len()).map(|_| Ok(())).collect();
                    }
                }

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
        arguments: EdgeNodePostExecutionArguments,
    ) -> Vec<Result<(), Error>> {
        init_logging();

        match (arguments.parent_type_name(), arguments.field_name()) {
            ("Query", "users") => {
                tracing::info!("Authorizing access to Query.users");

                #[derive(Debug, serde::Deserialize)]
                struct User {
                    id: usize,
                }

                let metadata: Metadata = arguments.metadata().unwrap_or_default();

                let nodes = match arguments.nodes() {
                    Ok(nodes) => nodes,
                    Err(_) => return vec![Err(contract_error())],
                };

                if let Some(role) = metadata.allow_role {
                    if context.get("role") == Some(role.clone()) {
                        tracing::info!("Granting access to role {role}");
                        return (0..nodes.len()).map(|_| Ok(())).collect();
                    }
                }

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
