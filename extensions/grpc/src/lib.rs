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
mod proto;

use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext, Subscription,
    host_io::http::{HttpMethod, HttpRequest, HttpResponse},
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};
use std::collections::HashMap;
use types::{Grpc, GrpcEndpoint, GrpcEndpointArgs, GrpcMethod, MethodType};

#[derive(ResolverExtension)]
struct GrpcExtension {
    endpoints: Vec<GrpcEndpoint>,
    // Map of address to fully qualified URL
    server_urls: HashMap<String, String>,
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
            server_urls: HashMap::new(),
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

    fn get_or_create_server_url(&mut self, address: &str) -> Result<String, Error> {
        if let Some(url) = self.server_urls.get(address) {
            return Ok(url.clone());
        }

        // Ensure the address has a scheme prefix (http or https)
        let url = if address.starts_with("http://") || address.starts_with("https://") {
            address.to_string()
        } else {
            // Default to HTTP if no scheme is specified
            format!("http://{}", address)
        };

        self.server_urls.insert(address.to_string(), url.clone());
        Ok(url)
    }


    // Helper for creating HTTP/2 gRPC requests
    fn create_http2_grpc_request(&self, url: &str, service: &str, method: &str, message: Vec<u8>) -> HttpRequest {
        // Create the path for the gRPC request
        let path = format!("/{}/{}", service, method);
        
        let frame = proto::framing::create_grpc_frame(&message);
        
        // Build the HTTP request with the appropriate headers for gRPC
        let mut request = HttpRequest::new(url, HttpMethod::POST);
        
        // Set required HTTP/2 pseudo-headers
        request.set_header(":scheme", "http");
        request.set_header(":path", &path);
        request.set_header(":method", "POST");
        request.set_header(":authority", url.split("://").nth(1).unwrap_or(url));
        
        // Set standard gRPC headers
        request.set_header("content-type", "application/grpc");
        request.set_header("te", "trailers");
        request.set_header("grpc-accept-encoding", "identity");
        request.set_header("user-agent", "grafbase-grpc-extension/1.0");
        
        // Set the body to the framed message
        request.set_body(frame);
        
        request
    }
    
    // Helper to make HTTP/2 gRPC request with proper framing
    async fn make_http2_grpc_request(
        &self,
        endpoint: &GrpcEndpoint,
        service: &str,
        method_name: &str,
        message: Vec<u8>,
    ) -> Result<HttpResponse, Error> {
        // Get or create the server URL
        let url = self.get_or_create_server_url(&endpoint.args.address)?;
        
        // Create the HTTP/2 request
        let request = self.create_http2_grpc_request(&url, service, method_name, message);
        
        // Execute the request
        let response = grafbase_sdk::host_io::http::execute(request)
            .await
            .map_err(|e| format!("Failed to execute gRPC request: {}", e))?;
        
        // Check for gRPC-specific error headers
        if let Some(grpc_status) = response.header("grpc-status") {
            if grpc_status != "0" {
                let message = response.header("grpc-message").unwrap_or("Unknown gRPC error");
                return Err(format!("gRPC error: status={}, message={}", grpc_status, message).into());
            }
        }
        
        Ok(response)
    }
    
    // Make a gRPC request, handling both unary and streaming methods
    async fn make_grpc_request(
        &self,
        endpoint: &GrpcEndpoint,
        method: &GrpcMethod,
        request_data: serde_json::Value,
    ) -> Result<serde_json::Value, Error> {
        // Convert request_data to bytes using protobuf serialization
        // In a real implementation, we'd use proper protobuf encoding
        // For this implementation, we'll use JSON serialization as a placeholder
        // Convert request_data to bytes using our proto module's JSON-to-binary encoding
        let message = proto::encoding::encode_json_to_binary(&request_data)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
            
        // Handle different method types
        match method.method_type {
            MethodType::Unary => {
                self.call_unary_method(endpoint, &method.name, &endpoint.args.service, message).await
            }
            MethodType::ServerStreaming => {
                self.call_server_streaming_method(endpoint, &method.name, &endpoint.args.service, message).await
            }
            _ => Err("Unsupported gRPC method type. Only unary and server streaming are supported.".into()),
        }
    }
    
    // Handle a unary gRPC method
    async fn call_unary_method(
        &self,
        endpoint: &GrpcEndpoint,
        method_name: &str,
        service: &str,
        message: Vec<u8>,
    ) -> Result<serde_json::Value, Error> {
        // Make the HTTP/2 request
        let response = self.make_http2_grpc_request(endpoint, service, method_name, message).await?;
        
        // Get the response body
        let response_bytes = response.body();
        
        // Parse gRPC frames from the response
        // Parse gRPC frames from the response
        let frames = proto::framing::extract_grpc_frames(&response_bytes)?;
        if frames.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        
        // For unary methods, we expect exactly one frame
        let response_data = &frames[0];
        
        // Convert the binary message to JSON
        // Convert the binary message to JSON using our proto module
        let json = proto::encoding::decode_binary_to_json(response_data)
            .map_err(|e| format!("Failed to deserialize response: {}", e))?;
        Ok(json)
    }
    
    // Handle a server-streaming gRPC method
    async fn call_server_streaming_method(
        &self,
        endpoint: &GrpcEndpoint,
        method_name: &str,
        service: &str,
        message: Vec<u8>,
    ) -> Result<serde_json::Value, Error> {
        // Make the HTTP/2 request
        let response = self.make_http2_grpc_request(endpoint, service, method_name, message).await?;
        
        // Get the response body
        let response_bytes = response.body();
        
        // Parse gRPC frames from the response
        // Parse gRPC frames from the response
        let frames = proto::framing::extract_grpc_frames(&response_bytes)?;
        // Convert each frame to JSON and collect into an array
        let mut results = Vec::new();
        for frame in frames {
            // Convert the binary message to JSON
            // Convert the binary message to JSON using our proto module
            let json = proto::encoding::decode_binary_to_json(&frame)
                .map_err(|e| format!("Failed to deserialize response frame: {}", e))?;
            results.push(json);
        }
        
        // Return the array of results
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

