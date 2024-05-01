use std::collections::BTreeMap;

use registry_v2::{
    resolvers::{http, variable_resolve_definition::VariableResolveDefinition, Resolver},
    FederationEntity, FederationKey, FederationResolver,
};

use super::OutputFieldKind;
use crate::graph::{Arity, DebugNode, OpenApiGraph, OutputType, Resource, ResourceOperation};

/// Creates all the federation entities for our API
pub fn federation_entities(graph: &OpenApiGraph) -> BTreeMap<String, FederationEntity> {
    graph
        .resources()
        .into_iter()
        .filter_map(|resource| Some((resource.name(graph)?, entity_for_resource(resource, graph)?)))
        .collect()
}

fn entity_for_resource(resource: Resource, graph: &OpenApiGraph) -> Option<FederationEntity> {
    let underlying_type = resource.underlying_type(graph)?;
    if !underlying_type.is_object() {
        // Only objects can be entities
        tracing::debug!(
            "Skipping {:?} because it does not represent an object",
            resource.debug(graph)
        );
        return None;
    }

    let unary_operations = resource
        .query_operations(graph)
        .filter(|operation| matches!(operation.arity, Arity::One))
        .filter(|operation| {
            // We're just working with path parameters now so skip operations that have none
            !operation.operation.path_parameters(graph).is_empty()
        })
        .filter(|operation| {
            // We maybe _could_ support operations w/ request bodies but
            // for simplicities sake I'm going to skip
            let request_body = operation.operation.request_body(graph);
            if request_body.is_some() {
                tracing::debug!(
                    "Skipping {:?} because it has a request body",
                    operation.operation.name(graph)
                );
            }

            request_body.is_none()
        })
        .collect::<Vec<_>>();

    if unary_operations.is_empty() {
        tracing::debug!(
            "Skipping {} because it has no unary operations: {:#?}",
            resource.name(graph).unwrap_or_default(),
            resource.debug(graph)
        );
        return None;
    }

    let keys = unary_operations
        .into_iter()
        .filter_map(|operation| key_for_operation(underlying_type, operation, graph))
        .collect::<Vec<_>>();

    if keys.is_empty() {
        tracing::debug!("Skipping {:?} because it has no keys", resource.name(graph));
        return None;
    }

    Some(FederationEntityBuilder::new().with_keys(keys).build())
}

pub struct FederationEntityBuilder(FederationEntity);

impl FederationEntityBuilder {
    pub fn new() -> Self {
        FederationEntityBuilder(FederationEntity::default())
    }
}

impl FederationEntityBuilder {
    pub fn with_keys(mut self, keys: Vec<FederationKey>) -> Self {
        self.0.keys.extend(keys);
        self
    }

    pub fn build(self) -> FederationEntity {
        self.0
    }
}

fn key_for_operation(
    underlying_type: OutputType,
    operation: ResourceOperation,
    graph: &OpenApiGraph,
) -> Option<FederationKey> {
    let path_parameters = operation.operation.path_parameters(graph);

    if !path_parameters.iter().copied().all(|parameter| {
        let field = underlying_type.field(parameter.openapi_name(graph), graph);

        field
            .map(|field| {
                !field.ty.is_list()
                    && matches!(
                        field.ty.inner_kind(graph),
                        OutputFieldKind::Enum | OutputFieldKind::Scalar
                    )
            })
            .unwrap_or_default()
    }) {
        // We're going to struggle to map the fields into the URL if they
        // don't exist or aren't simple enums & scalars.
        // So just skip generating a key for this field
        tracing::debug!(
            "Skipping {:?} because its keys are difficult: {:#?}",
            operation.operation.name(graph),
            path_parameters.debug(graph)
        );
        return None;
    }

    // For now I'm assuming all query parameters are optional.
    // Knowing OpenAPI that assumption will come back to bite me, but
    // I'll postpone that pain till it happens

    let resolver = operation
        .operation
        .http_resolver(graph, operation.federation_path_parameters(graph), vec![])?;

    let Resolver::Http(resolver) = resolver else {
        unreachable!();
    };

    Some(FederationKey::multiple(
        path_parameters
            .into_iter()
            .map(|param| param.graphql_name(graph).to_string())
            .collect(),
        FederationResolver::Http(resolver),
    ))
}

impl ResourceOperation {
    /// An operations http::PathParameters for its federation resolver
    ///
    /// This should be the same as it's http_path_parameters but it'll take data
    /// from the last_resolver_value instead of a GraphQL input
    fn federation_path_parameters(&self, graph: &OpenApiGraph) -> Vec<http::PathParameter> {
        self.operation
            .path_parameters(graph)
            .iter()
            .map(|param| {
                let name = param.openapi_name(graph).to_string();
                let input_name = param.graphql_name(graph).to_string();

                let input_value_type = param
                    .input_value(graph)
                    .and_then(|value| value.to_input_value_type(graph));

                // We _should_ always have an InputValueType, but if not just use the
                // input_name without applying transforms
                let variable_resolve_definition = match input_value_type {
                    None => VariableResolveDefinition::local_data(input_name),
                    Some(input_value_type) => {
                        VariableResolveDefinition::local_data_with_transforms(input_name, input_value_type.to_string())
                    }
                };

                http::PathParameter {
                    name,
                    variable_resolve_definition,
                }
            })
            .collect()
    }
}
