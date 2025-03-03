use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents a gRPC endpoint configuration
#[derive(Debug, Clone)]
pub struct GrpcEndpoint {
    /// Name of the gRPC service
    pub service_name: String,
    
    /// Address of the gRPC server (e.g., "localhost:50051")
    pub address: String,
    
    /// Methods available on this endpoint
    pub methods: HashMap<String, GrpcMethod>,
}

/// Represents a gRPC method configuration
#[derive(Debug, Clone)]
pub struct GrpcMethod {
    /// Name of the method
    pub name: String,
    
    /// Type of the method (unary, server streaming, etc.)
    pub method_type: MethodType,
}

/// Enum representing different types of gRPC methods
#[derive(Debug, Clone, PartialEq)]
pub enum MethodType {
    /// Unary RPC - single request, single response
    Unary,
    
    /// Server streaming RPC - single request, stream of responses
    ServerStreaming,
    
    // Client streaming and bidirectional streaming could be added here
    // ClientStreaming,
    // BidirectionalStreaming,
}

/// Arguments for the @grpc directive
#[derive(Debug, Deserialize)]
pub struct GrpcEndpointArgs {
    /// Name of the endpoint
    pub endpoint: String,
    
    /// Name of the gRPC service
    pub service: String,
    
    /// Address of the gRPC server
    pub address: String,
}

/// Arguments for the @method directive
#[derive(Debug, Deserialize)]
pub struct GrpcMethodArgs {
    /// Name of the endpoint this method belongs to
    pub endpoint: String,
    
    /// Name of the method
    pub name: String,
    
    /// Type of the method (unary, server_streaming, etc.)
    #[serde(default = "default_method_type")]
    pub method_type: String,
}

fn default_method_type() -> String {
    "unary".to_string()
}

/// Request payload for a gRPC call
#[derive(Debug, Serialize, Deserialize)]
pub struct GrpcRequest {
    /// Endpoint to call
    pub endpoint: String,
    
    /// Method to call
    pub method: String,
    
    /// Request payload as JSON
    pub payload: serde_json::Value,
}

/// Response from a gRPC call
#[derive(Debug, Serialize, Deserialize)]
pub struct GrpcResponse {
    /// Status of the call
    pub status: String,
    
    /// Response message
    pub message: String,
    
    /// Response payload as JSON
    pub payload: serde_json::Value,
}

use grafbase_sdk::{
    dynamic::{value::Value, Object},
    validation_error,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the type of gRPC method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodType {
    /// Unary RPC where the client sends a single request and gets a single response
    Unary,
    /// Server streaming RPC where the client sends a request and gets a stream of responses
    ServerStreaming,
    /// Client streaming RPC where the client sends a stream of requests and gets a single response
    ClientStreaming,
    /// Bidirectional streaming RPC where both sides send a stream of messages
    BidirectionalStreaming,
}

impl Default for MethodType {
    fn default() -> Self {
        MethodType::Unary
    }
}

/// Configuration for a gRPC method
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrpcMethod {
    /// The name of the gRPC method to call
    pub name: String,
    /// The type of gRPC method (unary, streaming, etc.)
    pub method_type: MethodType,
    /// Method-specific options
    pub options: HashMap<String, String>,
}

/// Arguments for defining a gRPC endpoint
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrpcEndpointArgs {
    /// The service name
    pub service: String,
    /// The server address (host:port)
    pub server: String,
    /// Whether to use TLS for secure communication
    pub tls: Option<bool>,
    /// Additional headers to send with every request
    pub headers: Option<HashMap<String, String>>,
    /// Timeout in seconds
    pub timeout: Option<u64>,
}

/// Configuration for a gRPC endpoint
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrpcEndpoint {
    /// The service name
    pub service: String,
    /// The server address (host:port)
    pub server: String,
    /// Whether to use TLS for secure communication
    pub tls: bool,
    /// Additional headers to send with every request
    pub headers: HashMap<String, String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Methods defined for this endpoint
    pub methods: HashMap<String, GrpcMethod>,
}

impl GrpcEndpoint {
    /// Creates a new gRPC endpoint from the provided arguments
    pub fn new(args: GrpcEndpointArgs) -> Self {
        Self {
            service: args.service,
            server: args.server,
            tls: args.tls.unwrap_or(false),
            headers: args.headers.unwrap_or_default(),
            timeout: args.timeout.unwrap_or(30),
            methods: HashMap::new(),
        }
    }

    /// Add a method to this endpoint
    pub fn add_method(&mut self, field_name: String, method: GrpcMethod) {
        self.methods.insert(field_name, method);
    }
}

/// Struct for parsing gRPC directive arguments
pub struct Grpc;

impl Grpc {
    /// Parse endpoint arguments from a directive
    pub fn parse_endpoint_args(value: &Value) -> Result<GrpcEndpointArgs, String> {
        let obj = value
            .as_object()
            .ok_or_else(|| validation_error!("@grpcService directive requires an object argument"))?;

        let service = obj
            .get("service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                validation_error!("`service` is required and must be a string in @grpcService")
            })?
            .to_string();

        let server = obj
            .get("server")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                validation_error!("`server` is required and must be a string in @grpcService")
            })?
            .to_string();

        let tls = obj.get("tls").and_then(|v| v.as_bool());
        
        let headers = obj.get("headers").and_then(|v| {
            v.as_object().map(|header_obj| {
                let mut headers = HashMap::new();
                for (key, value) in header_obj.iter() {
                    if let Some(value_str) = value.as_str() {
                        headers.insert(key.clone(), value_str.to_string());
                    }
                }
                headers
            })
        });

        let timeout = obj
            .get("timeout")
            .and_then(|v| v.as_u64())
            .or_else(|| obj.get("timeout").and_then(|v| v.as_i64()).map(|v| v as u64));

        Ok(GrpcEndpointArgs {
            service,
            server,
            tls,
            headers,
            timeout,
        })
    }

    /// Parse method arguments from a directive
    pub fn parse_method_args(value: &Value) -> Result<GrpcMethod, String> {
        let obj = value
            .as_object()
            .ok_or_else(|| validation_error!("@grpcMethod directive requires an object argument"))?;

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                validation_error!("`name` is required and must be a string in @grpcMethod")
            })?
            .to_string();

        let method_type = obj.get("type").and_then(|v| v.as_str()).map(|t| match t {
            "serverStreaming" => MethodType::ServerStreaming,
            "clientStreaming" => MethodType::ClientStreaming,
            "bidirectional" => MethodType::BidirectionalStreaming,
            _ => MethodType::Unary,
        }).unwrap_or_default();

        let options = obj.get("options").and_then(|v| {
            v.as_object().map(|opt_obj| {
                let mut options = HashMap::new();
                for (key, value) in opt_obj.iter() {
                    if let Some(value_str) = value.as_str() {
                        options.insert(key.clone(), value_str.to_string());
                    }
                }
                options
            })
        }).unwrap_or_default();

        Ok(GrpcMethod {
            name,
            method_type,
            options,
        })
    }
}

use async_graphql::dynamic::{FieldValue, TypeRef, Value};
use grafbase_sdk::dynamic::{Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Represents a gRPC service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcEndpoint {
    /// The address of the gRPC server (e.g., "localhost:50051")
    pub address: String,
    /// Optional timeout for requests in milliseconds
    pub timeout_ms: Option<u64>,
    /// Optional TLS configuration
    pub tls: Option<TlsConfig>,
    /// Optional headers to be sent with every request to this endpoint
    pub headers: Option<HashMap<String, String>>,
}

/// TLS configuration for secure gRPC connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to CA certificate
    pub ca_cert: Option<String>,
    /// Path to client certificate
    pub client_cert: Option<String>,
    /// Path to client key
    pub client_key: Option<String>,
    /// Domain override for verification
    pub domain: Option<String>,
}

/// Represents a gRPC method configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcMethod {
    /// Name of the service (e.g., "helloworld.Greeter")
    pub service: String,
    /// Name of the method (e.g., "SayHello")
    pub method: String,
    /// Type of the method (unary, server_streaming, etc.)
    pub method_type: MethodType,
    /// Optional input type transformation
    pub input_transform: Option<String>,
    /// Optional output type transformation
    pub output_transform: Option<String>,
}

/// Enum representing different types of gRPC methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodType {
    /// Unary RPC (one request, one response)
    Unary,
    /// Server streaming RPC (one request, stream of responses)
    ServerStreaming,
    /// Client streaming RPC (stream of requests, one response)
    ClientStreaming,
    /// Bidirectional streaming RPC (stream of requests, stream of responses)
    BidirectionalStreaming,
}

impl FromStr for MethodType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unary" => Ok(MethodType::Unary),
            "server_streaming" | "serverstreaming" => Ok(MethodType::ServerStreaming),
            "client_streaming" | "clientstreaming" => Ok(MethodType::ClientStreaming),
            "bidirectional_streaming" | "bidirectionalstreaming" | "bidi" => {
                Ok(MethodType::BidirectionalStreaming)
            }
            _ => Err(format!("Unknown method type: {}", s)),
        }
    }
}

impl fmt::Display for MethodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MethodType::Unary => write!(f, "unary"),
            MethodType::ServerStreaming => write!(f, "server_streaming"),
            MethodType::ClientStreaming => write!(f, "client_streaming"),
            MethodType::BidirectionalStreaming => write!(f, "bidirectional_streaming"),
        }
    }
}

/// Structure to hold gRPC arguments parsed from GraphQL directives
#[derive(Debug, Clone, Default)]
pub struct Grpc {
    /// The endpoint configuration
    pub endpoint: Option<GrpcEndpoint>,
    /// The method configuration
    pub method: Option<GrpcMethod>,
}

impl Grpc {
    /// Creates a new empty Grpc object
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes an endpoint directive argument
    pub fn process_endpoint(&mut self, args: HashMap<String, Value>) -> Result<(), String> {
        let address = args
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "gRPC endpoint address is required".to_string())?
            .to_string();

        let timeout_ms = args
            .get("timeoutMs")
            .and_then(|v| v.as_int())
            .map(|v| v as u64);

        let headers = args.get("headers").and_then(|v| {
            if let Value::Object(map) = v {
                let mut headers = HashMap::new();
                for (k, v) in map {
                    if let Some(value) = v.as_str() {
                        headers.insert(k.clone(), value.to_string());
                    }
                }
                Some(headers)
            } else {
                None
            }
        });

        // Parse TLS configuration if present
        let tls = if args.contains_key("tls") {
            if let Some(Value::Object(tls_map)) = args.get("tls") {
                let ca_cert = tls_map
                    .get("caCert")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let client_cert = tls_map
                    .get("clientCert")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let client_key = tls_map
                    .get("clientKey")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let domain = tls_map
                    .get("domain")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Some(TlsConfig {
                    ca_cert,
                    client_cert,
                    client_key,
                    domain,
                })
            } else {
                None
            }
        } else {
            None
        };

        self.endpoint = Some(GrpcEndpoint {
            address,
            timeout_ms,
            tls,
            headers,
        });

        Ok(())
    }

    /// Processes a method directive argument
    pub fn process_method(&mut self, args: HashMap<String, Value>) -> Result<(), String> {
        let service = args
            .get("service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "gRPC service name is required".to_string())?
            .to_string();

        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "gRPC method name is required".to_string())?
            .to_string();

        let method_type = args
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unary");

        let method_type = MethodType::from_str(method_type)
            .map_err(|e| format!("Invalid method type: {}", e))?;

        let input_transform = args
            .get("inputTransform")
            .and_then(|v| v.as_str())
            .map(String::from);

        let output_transform = args
            .get("outputTransform")
            .and_then(|v| v.as_str())
            .map(String::from);

        self.method = Some(GrpcMethod {
            service,
            method,
            method_type,
            input_transform,
            output_transform,
        });

        Ok(())
    }
}

/// Helper function to convert GraphQL values to JSON for gRPC requests
pub fn graphql_value_to_json(value: &Value, type_ref: Option<&TypeRef>) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                // Try to create a JSON number, fallback to string if it can't be represented
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::String(f.to_string()))
            } else {
                serde_json::Value::Null
            }
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::List(values) => {
            let element_type = type_ref.and_then(|t| t.list_element_type());
            serde_json::Value::Array(
                values
                    .iter()
                    .map(|v| graphql_value_to_json(v, element_type.as_ref()))
                    .collect(),
            )
        }
        Value::Object(map) => {
            let fields_type = type_ref.and_then(|t| {
                if t.is_input_object() {
                    Some(t)
                } else {
                    None
                }
            });

            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let field_type = fields_type.and_then(|t| t.input_field_type(k));
                obj.insert(k.clone(), graphql_value_to_json(v, field_type.as_ref()));
            }
            serde_json::Value::Object(obj)
        }
        Value::Enum(e) => serde_json::Value::String(e.clone()),
        _ => serde_json::Value::Null,
    }
}

/// Helper function to convert JSON values to GraphQL for gRPC responses
pub fn json_to_graphql_value(json: &serde_json::Value) -> FieldValue {
    match json {
        serde_json::Value::Null => FieldValue::Null,
        serde_json::Value::Bool(b) => FieldValue::Value(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                FieldValue::Value(Value::Number(n.as_i64().unwrap().into()))
            } else if n.is_f64() {
                FieldValue::Value(Value::Number(n.as_f64().unwrap().into()))
            } else {
                FieldValue::Null
            }
        }
        serde_json::Value::String(s) => FieldValue::Value(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr
                .iter()
                .filter_map(|v| match json_to_graphql_value(v) {
                    FieldValue::Value(val) => Some(val),
                    _ => None,
                })
                .collect();
            FieldValue::Value(Value::List(values))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let FieldValue::Value(val) = json_to_graphql_value(v) {
                    map.insert(k.clone(), val);
                }
            }
            FieldValue::Value(Value::Object(map))
        }
    }
}

/// Error types that can occur during gRPC operations
#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("Missing gRPC configuration: {0}")]
    MissingConfig(String),
    
    #[error("Invalid gRPC configuration: {0}")]
    InvalidConfig(String),
    
    #[error("gRPC transport error: {0}")]
    TransportError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("gRPC status error: code={code}, message={message}")]
    StatusError { code: i32, message: String },
}

impl From<GrpcError> for grafbase_sdk::Error {
    fn from(err: GrpcError) -> Self {
        grafbase_sdk::Error::new(err.to_string())
    }
}

/// Helper to build a gRPC request from a GraphQL request
pub fn build_grpc_request(request: &Request, method: &GrpcMethod) -> Result<serde_json::Value, GrpcError> {
    // Convert GraphQL arguments to JSON for gRPC
    let args = request.arguments();
    let input = serde_json::to_value(args)
        .map_err(|e| GrpcError::SerializationError(e.to_string()))?;
    
    // Apply input transformation if specified
    if let Some(transform) = &method.input_transform {
        // In a real implementation, this would apply the transformation
        // For now, we just pass through the input
        Ok(input)
    } else {
        Ok(input)
    }
}

/// Helper to build a GraphQL response from a gRPC response
pub fn build_graphql_response(
    grpc_response: serde_json::Value,
    method: &GrpcMethod,
) -> Result<Response, GrpcError> {
    // Apply output transformation if specified
    let transformed = if let Some(transform) = &method.output_transform {
        // In a real implementation, this would apply the transformation
        // For now, we just pass through the response
        grpc_response
    } else {
        grpc_response
    };
    
    // Convert JSON to GraphQL value
    let field_value = json_to_graphql_value(&transformed);
    
    match field_value {
        FieldValue::Value(value) => Ok(Response::new(value)),
        FieldValue::Null => Ok(Response::new(Value::Null)),
        _ => Err(GrpcError::DeserializationError(
            "Failed to convert gRPC response to GraphQL value".to_string(),
        )),
    }
}

use std::collections::HashMap;
use std::time::Duration;
use grafbase_sdk::{dynamic::serde_json, types::FieldDefinitionDirectiveArguments};
use serde::{Deserialize, Serialize};

/// Represents a gRPC endpoint configuration
#[derive(Debug, Clone)]
pub struct GrpcEndpoint {
    pub subgraph_name: String,
    pub args: GrpcEndpointArgs,
}

/// Arguments for a gRPC endpoint directive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcEndpointArgs {
    pub name: String,
    pub address: String,
    pub service: String,
    #[serde(default)]
    pub tls_enabled: bool,
    #[serde(default)]
    pub use_reflection: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

impl<'a> TryFrom<FieldDefinitionDirectiveArguments<'a>> for GrpcEndpointArgs {
    type Error = String;

    fn try_from(args: FieldDefinitionDirectiveArguments<'a>) -> Result<Self, Self::Error> {
        let name = args
            .get("name")
            .ok_or("Missing 'name' argument")?
            .to_string();
        let address = args
            .get("address")
            .ok_or("Missing 'address' argument")?
            .to_string();
        let service = args
            .get("service")
            .ok_or("Missing 'service' argument")?
            .to_string();
        
        // Optional arguments
        let tls_enabled = args
            .get("tlsEnabled")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        
        let use_reflection = args
            .get("useReflection")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        
        let timeout_ms = args
            .get("timeoutMs")
            .and_then(|v| v.as_u64());

        Ok(Self {
            name,
            address,
            service,
            tls_enabled,
            use_reflection,
            timeout_ms,
        })
    }
}

/// Represents a specific gRPC method
#[derive(Debug, Clone)]
pub struct GrpcMethod {
    pub name: String,
    pub method_type: MethodType,
}

/// Types of gRPC methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MethodType {
    Unary,
    ServerStreaming,
    ClientStreaming,
    BidirectionalStreaming,
}

/// Represents a gRPC directive on a field
pub struct Grpc<'a> {
    pub service: &'a str,
    pub method: &'a str,
    pub method_type: MethodType,
    pub timeout_ms: Option<u64>,
}

impl<'a> TryFrom<FieldDefinitionDirectiveArguments<'a>> for Grpc<'a> {
    type Error = String;

    fn try_from(args: FieldDefinitionDirectiveArguments<'a>) -> Result<Self, Self::Error> {
        let service = args
            .get("service")
            .ok_or("Missing 'service' argument")?
            .as_str()
            .ok_or("'service' argument must be a string")?;

        let method = args
            .get("method")
            .ok_or("Missing 'method' argument")?
            .as_str()
            .ok_or("'method' argument must be a string")?;

        let method_type_str = args
            .get("type")
            .map(|t| t.as_str().unwrap_or("unary"))
            .unwrap_or("unary");

        let method_type = match method_type_str {
            "unary" => MethodType::Unary,
            "server_streaming" => MethodType::ServerStreaming,
            "client_streaming" => MethodType::ClientStreaming,
            "bidirectional_streaming" => MethodType::BidirectionalStreaming,
            _ => return Err(format!("Invalid method type: {}", method_type_str)),
        };

        let timeout_ms = args
            .get("timeoutMs")
            .and_then(|v| v.as_u64());

        Ok(Self {
            service,
            method,
            method_type,
            timeout_ms,
        })
    }
}

/// Describes a Protobuf data type
#[derive(Debug, Clone)]
pub enum ProtobufType {
    String,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Float,
    Double,
    Bool,
    Bytes,
    Message(String), // Represents a complex message type with the type name
    Enum(String),    // Represents an enumeration with the enum name
    Repeated(Box<ProtobufType>),
    Map(Box<ProtobufType>, Box<ProtobufType>),
}

/// Descriptor for a gRPC method
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    pub name: String,
    pub fully_qualified_name: String,
    pub input_type: String,
    pub output_type: String,
    pub method_type: MethodType,
}

/// Descriptor for a gRPC service
#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub name: String,
    pub fully_qualified_name: String,
    pub methods: Vec<MethodDescriptor>,
}

/// Request data for a gRPC call
#
