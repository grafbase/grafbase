use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::{Anything, AuthorizationVerdict, AuthorizationVerdicts, AuthorizedHooks, EdgeDefinition, NodeDefinition},
};

use super::{guest_error_as_gql, Context, HooksWasi};

macro_rules! prepare_authorized {
    ($self:ident named $func_name:literal at $definition:expr; [$(($name:literal, $input:expr),)+]) => {{
        let Some(ref inner) = $self.0 else {
            return Err(PartialGraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
                PartialErrorCode::Unauthorized,
            ));
        };
        let instance = inner.authorization.get().await;
        let inputs = [$(
            encode($func_name, $definition, $name, $input)?,
        )+];
        (inner, instance, inputs)
    }};
}

fn encode<'a>(
    func_name: &str,
    definition: impl std::fmt::Display,
    name: &str,
    values: impl IntoIterator<Item: Anything<'a>>,
) -> Result<Vec<String>, PartialGraphqlError> {
    values
        .into_iter()
        .map(|value| {
            serde_json::to_string(&value).map_err(|_| {
                tracing::error!("{func_name} error at {definition}: failed to serialize {name}");
                PartialGraphqlError::internal_server_error()
            })
        })
        .collect()
}

impl AuthorizedHooks<Context> for HooksWasi {
    async fn authorize_edge_pre_execution<'a>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        arguments: impl Anything<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        let (inner, mut instance, [arguments, metadata]) = prepare_authorized!(
            self named "authorize_edge_pre_execution" at &definition;
            [("arguments", [arguments]), ("metadata", metadata),]
        );

        let arguments = arguments.into_iter().next().unwrap();
        let metadata = metadata.into_iter().next().unwrap_or_default();
        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        inner
            .run_and_measure(
                "authorize-edge-pre-execution",
                instance.authorize_edge_pre_execution(inner.shared_context(context), definition, arguments, metadata),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_edge_pre_execution error at: {error}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::Guest(error) => guest_error_as_gql(error, PartialErrorCode::Unauthorized),
            })?;

        Ok(())
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        context: &Context,
        definition: NodeDefinition<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        let (inner, mut instance, [metadata]) = prepare_authorized!(
            self named "authorize_node_pre_execution" at &definition;
            [ ("metadata", metadata),]
        );
        let metadata = metadata.into_iter().next().unwrap_or_default();
        let definition = wasi_component_loader::NodeDefinition {
            type_name: definition.type_name.to_string(),
        };

        inner
            .run_and_measure(
                "authorize-node-pre-execution",
                instance.authorize_node_pre_execution(inner.shared_context(context), definition, metadata),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_node_pre_execution error at: {error}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::Guest(error) => guest_error_as_gql(error, PartialErrorCode::Unauthorized),
            })?;

        Ok(())
    }

    async fn authorize_node_post_execution<'a>(
        &self,
        _context: &Context,
        definition: NodeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        let (_inner, mut _instance, [_nodes, metadata]) = prepare_authorized!(
            self named "authorize_node_post_execution" at &definition;
            [("nodes", nodes), ("metadata", metadata),]
        );
        let _metadata = metadata.into_iter().next().unwrap_or_default();
        let _definition = wasi_component_loader::NodeDefinition {
            type_name: definition.type_name.to_string(),
        };

        todo!()
    }

    async fn authorize_parent_edge_post_execution<'a>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        parents: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        let (inner, mut instance, [parents, metadata]) = prepare_authorized!(
            self named "authorize_parent_edge_post_execution" at &definition;
            [("parents", parents), ("metadata", metadata),]
        );
        let metadata = metadata.into_iter().next().unwrap_or_default();
        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        let results = inner
            .run_and_measure_multi_error(
                "authorize-parent-edge-post-execution",
                instance.authorize_parent_edge_post_execution(
                    inner.shared_context(context),
                    definition,
                    parents,
                    metadata,
                ),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_parent_edge_post_execution error at: {error}");
                    PartialGraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::Guest(error) => guest_error_as_gql(error, PartialErrorCode::Unauthorized),
            })?
            .into_iter()
            .map(|result| match result {
                Ok(()) => Ok(()),
                Err(error) => Err(guest_error_as_gql(error, PartialErrorCode::Unauthorized)),
            })
            .collect();

        Ok(results)
    }

    async fn authorize_edge_node_post_execution<'a>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        let (inner, mut instance, [nodes, metadata]) = prepare_authorized!(
            self named "authorize_edge_node_post_execution" at &definition;
            [("nodes", nodes), ("metadata", metadata),]
        );
        let metadata = metadata.into_iter().next().unwrap_or_default();
        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        let result = inner
            .run_and_measure_multi_error(
                "authorize-edge-node-post-execution",
                instance.authorize_edge_node_post_execution(inner.shared_context(context), definition, nodes, metadata),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_edge_node_post_execution error at: {error}");
                    PartialGraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::Guest(error) => guest_error_as_gql(error, PartialErrorCode::Unauthorized),
            })?
            .into_iter()
            .map(|result| match result {
                Ok(()) => Ok(()),
                Err(error) => Err(guest_error_as_gql(error, PartialErrorCode::Unauthorized)),
            })
            .collect();

        Ok(result)
    }

    async fn authorize_edge_post_execution<'a, Parent, Nodes>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        edges: impl IntoIterator<Item = (Parent, Nodes)> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts
    where
        Parent: Anything<'a>,
        Nodes: IntoIterator<Item: Anything<'a>> + Send,
    {
        let (inner, mut instance, [metadata]) = prepare_authorized!(
            self named "authorize_edge_post_execution" at &definition;
            [("metadata", metadata),]
        );

        let metadata: String = metadata.into_iter().next().unwrap_or_default();

        let edges: Vec<(String, Vec<String>)> = edges
            .into_iter()
            .map(|(parent, nodes): (Parent, Nodes)| {
                let parent = serde_json::to_string(&parent).map_err(|_| {
                    tracing::error!("authorize_edge_post_execution error at {definition}: failed to serialize edge");
                    PartialGraphqlError::internal_server_error()
                })?;
                let nodes = nodes
                    .into_iter()
                    .map(|node| {
                        serde_json::to_string(&node).map_err(|_| {
                            tracing::error!(
                                "authorize_edge_post_execution error at {definition}: failed to serialize edge"
                            );
                            PartialGraphqlError::internal_server_error()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok((parent, nodes))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        let result = inner
            .run_and_measure_multi_error(
                "authorize-edge-post-execution",
                instance.authorize_edge_post_execution(inner.shared_context(context), definition, edges, metadata),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_edge_post_execution error at: {error}");
                    PartialGraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::Guest(error) => guest_error_as_gql(error, PartialErrorCode::Unauthorized),
            })?
            .into_iter()
            .map(|result| match result {
                Ok(()) => Ok(()),
                Err(error) => Err(guest_error_as_gql(error, PartialErrorCode::Unauthorized)),
            })
            .collect();

        Ok(result)
    }
}
