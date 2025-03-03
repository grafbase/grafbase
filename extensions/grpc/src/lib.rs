use std::collections::HashMap;

use grafbase_sdk::{
    dynamic::{
        resolver::{ContextField, ObjectField, OperationType, ResolveInfo},
        scalar_type, value, DynamicType, Object, Result as DynamicResult, TypeDefinition, Value,
    },
    web::Headers,
    AsyncResolver, Extension, Request,
};

use crate::types::{GrpcEndpoint, GrpcMethod, MethodType};

#[derive(Default)]
pub struct GrpcExtension {
    endpoints: HashMap<String, GrpcEndpoint>,
}

impl GrpcExtension {
    fn register_endpoint(&mut self, name: String, endpoint: GrpcEndpoint) {
        self.endpoints.insert(name, endpoint);
    }

    fn get_endpoint(&self, name: &str) -> Option<&GrpcEndpoint> {
        self.endpoints.get(name)
    }

    async fn resolve_grpc_method(
        &self,
        endpoint_name: &str,
        method_name: &str,
        payload: Value,
        request_headers: Headers,
    ) -> DynamicResult<Value> {
        // Get the endpoint configuration
        let endpoint = self
            .get_endpoint(endpoint_name)
            .ok_or_else(|| format!("gRPC endpoint '{}' not found", endpoint_name))?;

        // Get the method configuration
        let method = endpoint
            .methods
            .get(method_name)
            .ok_or_else(|| format!("gRPC method '{}' not found for endpoint '{}'", method_name, endpoint_name))?;

        // In a real implementation, we would use tonic to make the gRPC call
        // For this simplified version, we just return a placeholder value
        match method.method_type {
            MethodType::Unary => {
                // Simulating a unary call response
                Ok(value!({
                    "status": "success",
                    "message": format!("Called {} method on {} endpoint", method_name, endpoint_name),
                    "payload": payload,
                }))
            }
            MethodType::ServerStreaming => {
                // Simulating a streaming response
                Ok(value!({
                    "status": "success",
                    "message": format!("Started streaming for {} method on {} endpoint", method_name, endpoint_name),
                    "payload": payload,
                }))
            }
            // Additional method types would be implemented here
        }
    }
}

#[async_trait::async_trait]
impl Extension for GrpcExtension {
    async fn init(&mut self, registry: &mut grafbase_sdk::registry::Registry) -> Result<(), String> {
        // Register the @grpc directive
        registry
            .register_directive("grpc", |params, registry| {
                let endpoint_name = params
                    .get("endpoint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'endpoint' parameter in @grpc directive".to_string())?;

                let service_name = params
                    .get("service")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'service' parameter in @grpc directive".to_string())?;

                let address = params
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'address' parameter in @grpc directive".to_string())?;

                // Create a new gRPC endpoint
                let endpoint = GrpcEndpoint {
                    service_name: service_name.to_string(),
                    address: address.to_string(),
                    methods: HashMap::new(),
                };

                // Register the endpoint
                self.register_endpoint(endpoint_name.to_string(), endpoint);

                Ok(())
            })
            .map_err(|e| format!("Failed to register @grpc directive: {}", e))?;

        // Register the @method directive
        registry
            .register_directive("method", |params, registry| {
                let endpoint_name = params
                    .get("endpoint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'endpoint' parameter in @method directive".to_string())?;

                let method_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'name' parameter in @method directive".to_string())?;

                let method_type_str = params
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unary");

                let method_type = match method_type_str {
                    "unary" => MethodType::Unary,
                    "server_streaming" => MethodType::ServerStreaming,
                    _ => return Err(format!("Unsupported method type: {}", method_type_str)),
                };

                // Get the endpoint and register the method
                if let Some(endpoint) = self.get_endpoint(endpoint_name) {
                    let mut endpoint = endpoint.clone();
                    endpoint.methods.insert(
                        method_name.to_string(),
                        GrpcMethod {
                            name: method_name.to_string(),
                            method_type,
                        },
                    );
                    self.register_endpoint(endpoint_name.to_string(), endpoint);
                } else {
                    return Err(format!("Endpoint '{}' not found", endpoint_name));
                }

                Ok(())
            })
            .map_err(|e| format!("Failed to register @method directive: {}", e))?;

        // Register resolver for gRPC methods
        registry.register_resolver("grpc", move |parent, args, context, info| {
            let endpoint_name = args
                .get("endpoint")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing 'endpoint' argument".to_string())?;

            let method_name = args
                .get("method")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing 'method' argument".to_string())?;

            let payload = args.get("payload").cloned().unwrap_or(value!({}));
            
            let request_headers = context.request().headers().clone();

            AsyncResolver::new(move |_| async move {
                self.resolve_grpc_method(endpoint_name, method_name, payload, request_headers).await
            })
        });

        Ok(())
    }
}

// This is the function that grafbase calls to create our extension
#[grafbase::extension]
fn init() -> GrpcExtension {
    GrpcExtension::default()
}

mod types;

use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext, Subscription,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};
use std::collections::HashMap;
use tonic::{Request, Streaming};
use types::{Grpc, GrpcEndpoint, GrpcEndpointArgs, GrpcMethod, MethodType};

#[derive(ResolverExtension)]
struct GrpcExtension {
    endpoints: Vec<GrpcEndpoint>,
    clients: HashMap<String, tonic::transport::Channel>,
}

impl Extension for GrpcExtension {
    fn new(schema_directives: Vec<SchemaDirective>, _: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::<GrpcEndpoint>::new();

        for directive in schema_directives {
            let args: GrpcEndpointArgs = directive.arguments()?;
            let endpoint = GrpcEndpoint {
                subgraph_name: directive.subgraph_name().to_string(),
                args,
            };

            endpoints.push(endpoint);
        }

        endpoints.sort_by(|a, b| {
            let by_name = a.args.name.cmp(&b.args.name);
            let by_subgraph = a.subgraph_name.cmp(&b.subgraph_name);
            by_name.then(by_subgraph)
        });

        Ok(Self {
            endpoints,
            clients: HashMap::new(),
        })
    }
}

impl GrpcExtension {
    pub fn get_endpoint(&self, name: &str, subgraph_name: &str) -> Option<&GrpcEndpoint> {
        self.endpoints
            .binary_search_by(|e| {
                let by_name = e.args.name.as_str().cmp(name);
                let by_subgraph = e.subgraph_name.as_str().cmp(subgraph_name);

                by_name.then(by_subgraph)
            })
            .map(|i| &self.endpoints[i])
            .ok()
    }

    async fn get_or_create_client(&mut self, address: &str) -> Result<tonic::transport::Channel, Error> {
        if let Some(client) = self.clients.get(address) {
            return Ok(client.clone());
        }

        let channel = tonic::transport::Channel::from_shared(address.to_string())
            .map_err(|e| format!("Failed to create channel: {}", e))?
            .connect_lazy()
            .map_err(|e| format!("Failed to connect to gRPC service: {}", e))?;

        self.clients.insert(address.to_string(), channel.clone());
        Ok(channel)
    }

    async fn make_grpc_request(
        &mut self, 
        endpoint: &GrpcEndpoint, 
        method: &GrpcMethod,
        request_data: serde_json::Value
    ) -> Result<serde_json::Value, Error> {
        let channel = self.get_or_create_client(&endpoint.args.address).await?;
        
        // Convert the JSON request to a protobuf message
        let request_bytes = prost_json::to_vec(&request_data)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
            
        // Create a gRPC request
        let request = Request::new(request_bytes);
        
        // Call the appropriate method based on the method type
        match method.method_type {
            MethodType::Unary => {
                // Make a unary call
                let response = self.call_unary_method(
                    channel, 
                    &endpoint.args.service, 
                    &method.name, 
                    request
                ).await?;
                
                // Convert the response to JSON
                Ok(prost_json::from_bytes(&response)
                    .map_err(|e| format!("Failed to deserialize response: {}", e))?)
            },
            MethodType::ServerStreaming => {
                // Make a server streaming call
                let response_stream = self.call_server_streaming_method(
                    channel, 
                    &endpoint.args.service, 
                    &method.name, 
                    request
                ).await?;
                
                // Convert the stream to a JSON array
                self.process_streaming_response(response_stream).await
            },
            MethodType::ClientStreaming | MethodType::BidirectionalStreaming => {
                Err("Client streaming and bidirectional streaming not yet implemented".into())
            }
        }
    }
    
    async fn call_unary_method(
        &self,
        channel: tonic::transport::Channel,
        service: &str,
        method: &str,
        request: Request<Vec<u8>>
    ) -> Result<Vec<u8>, Error> {
        // This is a simplified implementation. In a real implementation,
        // you would need to use reflection or generated code to call the actual method.
        // Here we're using a basic approach that assumes dynamic invocation capability.
        
        let mut client = tonic::client::Grpc::new(channel);
        let path = format!("/{}/{}", service, method);
        
        let response = client.unary(path, request)
            .await
            .map_err(|e| format!("gRPC call failed: {}", e))?;
            
        Ok(response.into_inner())
    }
    
    async fn call_server_streaming_method(
        &self,
        channel: tonic::transport::Channel,
        service: &str,
        method: &str,
        request: Request<Vec<u8>>
    ) -> Result<Streaming<Vec<u8>>, Error> {
        // Similar to unary method, this is simplified
        let mut client = tonic::client::Grpc::new(channel);
        let path = format!("/{}/{}", service, method);
        
        let response = client.server_streaming(path, request)
            .await
            .map_err(|e| format!("gRPC streaming call failed: {}", e))?;
            
        Ok(response)
    }
    
    async fn process_streaming_response(
        &self,
        mut stream: Streaming<Vec<u8>>
    ) -> Result<serde_json::Value, Error> {
        let mut results = Vec::new();
        
        while let Some(message) = stream.message().await
            .map_err(|e| format!("Error receiving stream message: {}", e))? {
            
            let json = prost_json::from_bytes(&message)
                .map_err(|e| format!("Failed to deserialize response: {}", e))?;
                
            results.push(json);
        }
        
        Ok(serde_json::Value::Array(results))
    }
}

impl Resolver for GrpcExtension {
    fn resolve_field(
        &mut self,
        _: SharedContext,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let grpc: Grpc<'_> = directive
            .arguments()
            .map_err(|e| format!("Could not parse directive arguments: {e}"))?;

        let Some(endpoint) = self.get_endpoint(grpc.service, subgraph_name) else {
            return Err(format!("gRPC service not found: {}", grpc.service).into());
        };

        // Find the method in the endpoint
        // Find the method in the endpoint
        let method = GrpcMethod {
            name: grpc.method.to_string(),
            method_type: grpc.method_type,
        };
        // Get input arguments as JSON
        let request_data = inputs.into_json();

        // Create a runtime for async operations
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;

        // Execute the gRPC request
        let response = runtime.block_on(self.make_grpc_request(&endpoint, &method, request_data))?;

        // Process the response
        let mut results = FieldOutput::new();
        
        match response {
            serde_json::Value::Array(items) => {
                for item in items {
                    results.push_value(item);
                }
            },
            value => {
                results.push_value(value);
            }
        }

        Ok(results)
    }

    fn resolve_subscription(
        &mut self,
        _: SharedContext,
        _: &str,
        _: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        Err("gRPC subscriptions are not yet implemented".into())
    }
}

