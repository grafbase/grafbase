/// Error thrown when accessing the headers. Headers names or values
/// must not contain any special characters.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum HeaderError {
    /// the given header value is not valid
    InvalidHeaderValue,
    /// the given header name is not valid
    InvalidHeaderName,
}
impl HeaderError {
    pub fn name(&self) -> &'static str {
        match self {
            HeaderError::InvalidHeaderValue => "invalid-header-value",
            HeaderError::InvalidHeaderName => "invalid-header-name",
        }
    }
    pub fn message(&self) -> &'static str {
        match self {
            HeaderError::InvalidHeaderValue => "the given header value is not valid",
            HeaderError::InvalidHeaderName => "the given header name is not valid",
        }
    }
}
impl ::core::fmt::Debug for HeaderError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HeaderError")
            .field("code", &(*self as i32))
            .field("name", &self.name())
            .field("message", &self.message())
            .finish()
    }
}
impl ::core::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{} (error {})", self.name(), * self as i32)
    }
}
impl std::error::Error for HeaderError {}
impl HeaderError {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> HeaderError {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => HeaderError::InvalidHeaderValue,
            1 => HeaderError::InvalidHeaderName,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
/// Error variant sent if failing to write to access log.
#[derive(Clone)]
pub enum LogError {
    /// The log channel is over capacity. The data is returned to the caller.
    ChannelFull(_rt::Vec<u8>),
    /// The channel is closed.
    ChannelClosed,
}
impl ::core::fmt::Debug for LogError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            LogError::ChannelFull(e) => {
                f.debug_tuple("LogError::ChannelFull").field(e).finish()
            }
            LogError::ChannelClosed => f.debug_tuple("LogError::ChannelClosed").finish(),
        }
    }
}
impl ::core::fmt::Display for LogError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for LogError {}
/// A context object is available in all hooks during the whole request
/// lifecycle. It can be used to store custom data in one hook and make it
/// available in the hooks executed later in the request.
///
/// This resource provides mutable access to the context and is available only
/// in the gateway request hook.
#[derive(Debug)]
#[repr(transparent)]
pub struct Context {
    handle: _rt::Resource<Context>,
}
impl Context {
    #[doc(hidden)]
    pub unsafe fn from_handle(handle: u32) -> Self {
        Self {
            handle: _rt::Resource::from_handle(handle),
        }
    }
    #[doc(hidden)]
    pub fn take_handle(&self) -> u32 {
        _rt::Resource::take_handle(&self.handle)
    }
    #[doc(hidden)]
    pub fn handle(&self) -> u32 {
        _rt::Resource::handle(&self.handle)
    }
}
unsafe impl _rt::WasmResource for Context {
    #[inline]
    unsafe fn drop(_handle: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        unreachable!();
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[resource-drop]context"]
                fn drop(_: u32);
            }
            drop(_handle);
        }
    }
}
/// The context as a read-only object.
#[derive(Debug)]
#[repr(transparent)]
pub struct SharedContext {
    handle: _rt::Resource<SharedContext>,
}
impl SharedContext {
    #[doc(hidden)]
    pub unsafe fn from_handle(handle: u32) -> Self {
        Self {
            handle: _rt::Resource::from_handle(handle),
        }
    }
    #[doc(hidden)]
    pub fn take_handle(&self) -> u32 {
        _rt::Resource::take_handle(&self.handle)
    }
    #[doc(hidden)]
    pub fn handle(&self) -> u32 {
        _rt::Resource::handle(&self.handle)
    }
}
unsafe impl _rt::WasmResource for SharedContext {
    #[inline]
    unsafe fn drop(_handle: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        unreachable!();
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[resource-drop]shared-context"]
                fn drop(_: u32);
            }
            drop(_handle);
        }
    }
}
/// Provides access to the request headers. Available in a mutable form
/// only in the gateway request hook.
#[derive(Debug)]
#[repr(transparent)]
pub struct Headers {
    handle: _rt::Resource<Headers>,
}
impl Headers {
    #[doc(hidden)]
    pub unsafe fn from_handle(handle: u32) -> Self {
        Self {
            handle: _rt::Resource::from_handle(handle),
        }
    }
    #[doc(hidden)]
    pub fn take_handle(&self) -> u32 {
        _rt::Resource::take_handle(&self.handle)
    }
    #[doc(hidden)]
    pub fn handle(&self) -> u32 {
        _rt::Resource::handle(&self.handle)
    }
}
unsafe impl _rt::WasmResource for Headers {
    #[inline]
    unsafe fn drop(_handle: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        unreachable!();
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[resource-drop]headers"]
                fn drop(_: u32);
            }
            drop(_handle);
        }
    }
}
/// Defines an edge in a type
#[derive(Clone)]
pub struct EdgeDefinition {
    /// The name of the type the edge is part of
    pub parent_type_name: _rt::String,
    /// The name of the field of this edge
    pub field_name: _rt::String,
}
impl ::core::fmt::Debug for EdgeDefinition {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("EdgeDefinition")
            .field("parent-type-name", &self.parent_type_name)
            .field("field-name", &self.field_name)
            .finish()
    }
}
/// Defines a node
#[derive(Clone)]
pub struct NodeDefinition {
    /// The name of the type of this node
    pub type_name: _rt::String,
}
impl ::core::fmt::Debug for NodeDefinition {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("NodeDefinition").field("type-name", &self.type_name).finish()
    }
}
/// Info about an executed HTTP request.
#[derive(Clone)]
pub struct ExecutedHttpRequest {
    /// The request method.
    pub method: _rt::String,
    /// The request URL.
    pub url: _rt::String,
    /// The response status code.
    pub status_code: u16,
    /// The outputs of executed on-operation-response hooks for every operation of the request.
    pub on_operation_response_outputs: _rt::Vec<_rt::Vec<u8>>,
}
impl ::core::fmt::Debug for ExecutedHttpRequest {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ExecutedHttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("status-code", &self.status_code)
            .field("on-operation-response-outputs", &self.on_operation_response_outputs)
            .finish()
    }
}
/// An error returned from a field.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FieldError {
    /// The number of errors.
    pub count: u64,
    /// The returned data is null.
    pub data_is_null: bool,
}
impl ::core::fmt::Debug for FieldError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("FieldError")
            .field("count", &self.count)
            .field("data-is-null", &self.data_is_null)
            .finish()
    }
}
/// An error from a GraphQL request.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RequestError {
    /// The number of errors.
    pub count: u64,
}
impl ::core::fmt::Debug for RequestError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("RequestError").field("count", &self.count).finish()
    }
}
/// A status of a GraphQL operation.
#[derive(Clone, Copy)]
pub enum GraphqlResponseStatus {
    /// Request was successful.
    Success,
    /// A field returned an error.
    FieldError(FieldError),
    /// A request error.
    RequestError(RequestError),
    /// The request was refused.
    RefusedRequest,
}
impl ::core::fmt::Debug for GraphqlResponseStatus {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            GraphqlResponseStatus::Success => {
                f.debug_tuple("GraphqlResponseStatus::Success").finish()
            }
            GraphqlResponseStatus::FieldError(e) => {
                f.debug_tuple("GraphqlResponseStatus::FieldError").field(e).finish()
            }
            GraphqlResponseStatus::RequestError(e) => {
                f.debug_tuple("GraphqlResponseStatus::RequestError").field(e).finish()
            }
            GraphqlResponseStatus::RefusedRequest => {
                f.debug_tuple("GraphqlResponseStatus::RefusedRequest").finish()
            }
        }
    }
}
/// Info about an executed operation.
#[derive(Clone)]
pub struct ExecutedOperation {
    /// The name of the operation, if present.
    pub name: Option<_rt::String>,
    /// The operation document in sanitized form.
    pub document: _rt::String,
    /// The time taken in preparing.
    pub prepare_duration_ms: u64,
    /// True, if the plan was taken from cache.
    pub cached_plan: bool,
    /// Time in milliseconds spent executing the operation.
    pub duration_ms: u64,
    /// The status of the operation.
    pub status: GraphqlResponseStatus,
    /// If queried any subgraphs, the outputs of on-subgraph-response hooks.
    /// Will be empty if no subgraphs were called.
    pub on_subgraph_response_outputs: _rt::Vec<_rt::Vec<u8>>,
}
impl ::core::fmt::Debug for ExecutedOperation {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ExecutedOperation")
            .field("name", &self.name)
            .field("document", &self.document)
            .field("prepare-duration-ms", &self.prepare_duration_ms)
            .field("cached-plan", &self.cached_plan)
            .field("duration-ms", &self.duration_ms)
            .field("status", &self.status)
            .field("on-subgraph-response-outputs", &self.on_subgraph_response_outputs)
            .finish()
    }
}
/// Information on a response
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubgraphResponse {
    /// The milliseconds it took to connect to the host.
    pub connection_time_ms: u64,
    /// The milliseconds it took for the host to respond with data.
    pub response_time_ms: u64,
    /// The response status code
    pub status_code: u16,
}
impl ::core::fmt::Debug for SubgraphResponse {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("SubgraphResponse")
            .field("connection-time-ms", &self.connection_time_ms)
            .field("response-time-ms", &self.response_time_ms)
            .field("status-code", &self.status_code)
            .finish()
    }
}
/// Cache status of a subgraph call.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum CacheStatus {
    /// All data fetched from cache.
    Hit,
    /// Some data fetched from cache.
    PartialHit,
    /// Cache miss
    Miss,
}
impl ::core::fmt::Debug for CacheStatus {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            CacheStatus::Hit => f.debug_tuple("CacheStatus::Hit").finish(),
            CacheStatus::PartialHit => f.debug_tuple("CacheStatus::PartialHit").finish(),
            CacheStatus::Miss => f.debug_tuple("CacheStatus::Miss").finish(),
        }
    }
}
impl CacheStatus {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> CacheStatus {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => CacheStatus::Hit,
            1 => CacheStatus::PartialHit,
            2 => CacheStatus::Miss,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
/// Subgraph response variant.
#[derive(Clone, Copy)]
pub enum SubgraphRequestExecutionKind {
    /// Internal server error in the gateway.
    InternalServerError,
    /// Response prevented by subgraph request hook.
    HookError,
    /// HTTP request failed.
    RequestError,
    /// Request was rate-limited.
    RateLimited,
    /// A response was received.
    Response(SubgraphResponse),
}
impl ::core::fmt::Debug for SubgraphRequestExecutionKind {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            SubgraphRequestExecutionKind::InternalServerError => {
                f.debug_tuple("SubgraphRequestExecutionKind::InternalServerError")
                    .finish()
            }
            SubgraphRequestExecutionKind::HookError => {
                f.debug_tuple("SubgraphRequestExecutionKind::HookError").finish()
            }
            SubgraphRequestExecutionKind::RequestError => {
                f.debug_tuple("SubgraphRequestExecutionKind::RequestError").finish()
            }
            SubgraphRequestExecutionKind::RateLimited => {
                f.debug_tuple("SubgraphRequestExecutionKind::RateLimited").finish()
            }
            SubgraphRequestExecutionKind::Response(e) => {
                f.debug_tuple("SubgraphRequestExecutionKind::Response").field(e).finish()
            }
        }
    }
}
/// Info about an executed subgraph request.
#[derive(Clone)]
pub struct ExecutedSubgraphRequest {
    /// The name of the subgraph.
    pub subgraph_name: _rt::String,
    /// The request method.
    pub method: _rt::String,
    /// The subgraph URL.
    pub url: _rt::String,
    /// The subgraph responses
    pub executions: _rt::Vec<SubgraphRequestExecutionKind>,
    /// The cache status of the subgraph call.
    pub cache_status: CacheStatus,
    /// The time in milliseconds taken for the whole operation.
    pub total_duration_ms: u64,
    /// True, if the subgraph returned any errors.
    pub has_errors: bool,
}
impl ::core::fmt::Debug for ExecutedSubgraphRequest {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ExecutedSubgraphRequest")
            .field("subgraph-name", &self.subgraph_name)
            .field("method", &self.method)
            .field("url", &self.url)
            .field("executions", &self.executions)
            .field("cache-status", &self.cache_status)
            .field("total-duration-ms", &self.total_duration_ms)
            .field("has-errors", &self.has_errors)
            .finish()
    }
}
/// An error response can be used to inject an error to the GraphQL response.
#[derive(Clone)]
pub struct Error {
    /// Adds the given extensions to the response extensions. The first item in
    /// the tuple is the extension key, and the second item is the extension value.
    /// The extension value can be string-encoded JSON, which will be converted as
    /// JSON in the response. It can also be just a string, which will be converted as
    /// a JSON string in the response.
    pub extensions: _rt::Vec<(_rt::String, _rt::String)>,
    /// The error message.
    pub message: _rt::String,
}
impl ::core::fmt::Debug for Error {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Error")
            .field("extensions", &self.extensions)
            .field("message", &self.message)
            .finish()
    }
}
impl ::core::fmt::Display for Error {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
/// An HTTP error response.
#[derive(Clone)]
pub struct ErrorResponse {
    /// HTTP status code. Must be a valid status code. If not, the status code will be 500.
    pub status_code: u16,
    /// List of GraphQL errors.
    pub errors: _rt::Vec<Error>,
}
impl ::core::fmt::Debug for ErrorResponse {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ErrorResponse")
            .field("status-code", &self.status_code)
            .field("errors", &self.errors)
            .finish()
    }
}
impl ::core::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ErrorResponse {}
/// A HTTP client.
#[derive(Debug)]
#[repr(transparent)]
pub struct HttpClient {
    handle: _rt::Resource<HttpClient>,
}
impl HttpClient {
    #[doc(hidden)]
    pub unsafe fn from_handle(handle: u32) -> Self {
        Self {
            handle: _rt::Resource::from_handle(handle),
        }
    }
    #[doc(hidden)]
    pub fn take_handle(&self) -> u32 {
        _rt::Resource::take_handle(&self.handle)
    }
    #[doc(hidden)]
    pub fn handle(&self) -> u32 {
        _rt::Resource::handle(&self.handle)
    }
}
unsafe impl _rt::WasmResource for HttpClient {
    #[inline]
    unsafe fn drop(_handle: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        unreachable!();
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[resource-drop]http-client"]
                fn drop(_: u32);
            }
            drop(_handle);
        }
    }
}
/// A sender for the system access log.
#[derive(Debug)]
#[repr(transparent)]
pub struct AccessLog {
    handle: _rt::Resource<AccessLog>,
}
impl AccessLog {
    #[doc(hidden)]
    pub unsafe fn from_handle(handle: u32) -> Self {
        Self {
            handle: _rt::Resource::from_handle(handle),
        }
    }
    #[doc(hidden)]
    pub fn take_handle(&self) -> u32 {
        _rt::Resource::take_handle(&self.handle)
    }
    #[doc(hidden)]
    pub fn handle(&self) -> u32 {
        _rt::Resource::handle(&self.handle)
    }
}
unsafe impl _rt::WasmResource for AccessLog {
    #[inline]
    unsafe fn drop(_handle: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        unreachable!();
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[resource-drop]access-log"]
                fn drop(_: u32);
            }
            drop(_handle);
        }
    }
}
/// The HTTP method.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum HttpMethod {
    /// The GET method requests a representation of the specified resource. Requests using GET should only retrieve data.
    Get,
    /// The POST method is used to submit an entity to the specified resource, often causing a change in state or side effects on the server.
    Post,
    /// The PUT method replaces all current representations of the target resource with the request payload.
    Put,
    /// The DELETE method deletes the specified resource.
    Delete,
    /// The PATCH method is used to apply partial modifications to a resource.
    Patch,
    /// The HEAD method asks for a response identical to that of a GET request, but without the response body.
    Head,
    /// The OPTIONS method is used to describe the communication options for the target resource.
    Options,
    /// The CONNECT method establishes a tunnel to the server identified by the target resource.
    Connect,
    /// The TRACE method performs a message loop-back test along the path to the target resource.
    Trace,
}
impl ::core::fmt::Debug for HttpMethod {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            HttpMethod::Get => f.debug_tuple("HttpMethod::Get").finish(),
            HttpMethod::Post => f.debug_tuple("HttpMethod::Post").finish(),
            HttpMethod::Put => f.debug_tuple("HttpMethod::Put").finish(),
            HttpMethod::Delete => f.debug_tuple("HttpMethod::Delete").finish(),
            HttpMethod::Patch => f.debug_tuple("HttpMethod::Patch").finish(),
            HttpMethod::Head => f.debug_tuple("HttpMethod::Head").finish(),
            HttpMethod::Options => f.debug_tuple("HttpMethod::Options").finish(),
            HttpMethod::Connect => f.debug_tuple("HttpMethod::Connect").finish(),
            HttpMethod::Trace => f.debug_tuple("HttpMethod::Trace").finish(),
        }
    }
}
impl HttpMethod {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> HttpMethod {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => HttpMethod::Get,
            1 => HttpMethod::Post,
            2 => HttpMethod::Put,
            3 => HttpMethod::Delete,
            4 => HttpMethod::Patch,
            5 => HttpMethod::Head,
            6 => HttpMethod::Options,
            7 => HttpMethod::Connect,
            8 => HttpMethod::Trace,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
/// A HTTP request.
#[derive(Clone)]
pub struct HttpRequest {
    /// The HTTP method.
    pub method: HttpMethod,
    /// The URL to send the request to.
    pub url: _rt::String,
    /// The headers to send with the request. Keys and values must be ASCII strings.
    pub headers: _rt::Vec<(_rt::String, _rt::String)>,
    /// The body of the request. If the body is set, the Content-Type header must be set.
    pub body: _rt::Vec<u8>,
    /// The timeout in milliseconds for the request. If not set, no timeout is used.
    pub timeout_ms: Option<u64>,
}
impl ::core::fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .field("timeout-ms", &self.timeout_ms)
            .finish()
    }
}
/// The HTTP version.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum HttpVersion {
    /// The HTTP/0.9 version.
    Http09,
    /// The HTTP/1.0 version.
    Http10,
    /// The HTTP/1.1 version.
    Http11,
    /// The HTTP/2.0 version.
    Http20,
    /// The HTTP/3.0 version.
    Http30,
}
impl ::core::fmt::Debug for HttpVersion {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            HttpVersion::Http09 => f.debug_tuple("HttpVersion::Http09").finish(),
            HttpVersion::Http10 => f.debug_tuple("HttpVersion::Http10").finish(),
            HttpVersion::Http11 => f.debug_tuple("HttpVersion::Http11").finish(),
            HttpVersion::Http20 => f.debug_tuple("HttpVersion::Http20").finish(),
            HttpVersion::Http30 => f.debug_tuple("HttpVersion::Http30").finish(),
        }
    }
}
impl HttpVersion {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> HttpVersion {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => HttpVersion::Http09,
            1 => HttpVersion::Http10,
            2 => HttpVersion::Http11,
            3 => HttpVersion::Http20,
            4 => HttpVersion::Http30,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
/// An HTTP response.
#[derive(Clone)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub status: u16,
    /// The HTTP version.
    pub version: HttpVersion,
    /// The headers of the response.
    pub headers: _rt::Vec<(_rt::String, _rt::String)>,
    /// The body of the response.
    pub body: _rt::Vec<u8>,
}
impl ::core::fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HttpResponse")
            .field("status", &self.status)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}
/// An HTTP error.
#[derive(Clone)]
pub enum HttpError {
    /// The request timed out.
    Timeout,
    /// The request failed due to an error (invalid user data).
    Request(_rt::String),
    /// The request failed due to an error (server connection failed).
    Connect(_rt::String),
}
impl ::core::fmt::Debug for HttpError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            HttpError::Timeout => f.debug_tuple("HttpError::Timeout").finish(),
            HttpError::Request(e) => {
                f.debug_tuple("HttpError::Request").field(e).finish()
            }
            HttpError::Connect(e) => {
                f.debug_tuple("HttpError::Connect").field(e).finish()
            }
        }
    }
}
impl ::core::fmt::Display for HttpError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for HttpError {}
impl Context {
    #[allow(unused_unsafe, clippy::all)]
    /// Fetches a context value with the given name, if existing.
    pub fn get(&self, name: &str) -> Option<_rt::String> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]context.get"]
                fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
            let l2 = i32::from(*ptr1.add(0).cast::<u8>());
            match l2 {
                0 => None,
                1 => {
                    let e = {
                        let l3 = *ptr1.add(4).cast::<*mut u8>();
                        let l4 = *ptr1.add(8).cast::<usize>();
                        let len5 = l4;
                        let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                        _rt::string_lift(bytes5)
                    };
                    Some(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl Context {
    #[allow(unused_unsafe, clippy::all)]
    /// Stores a context value with the given name.
    pub fn set(&self, name: &str, value: &str) {
        unsafe {
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let vec1 = value;
            let ptr1 = vec1.as_ptr().cast::<u8>();
            let len1 = vec1.len();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]context.set"]
                fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8, _: usize);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8, _: usize) {
                unreachable!()
            }
            wit_import(
                (self).handle() as i32,
                ptr0.cast_mut(),
                len0,
                ptr1.cast_mut(),
                len1,
            );
        }
    }
}
impl Context {
    #[allow(unused_unsafe, clippy::all)]
    /// Deletes a context value with the given name. Returns the value
    /// if existing.
    pub fn delete(&self, name: &str) -> Option<_rt::String> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]context.delete"]
                fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
            let l2 = i32::from(*ptr1.add(0).cast::<u8>());
            match l2 {
                0 => None,
                1 => {
                    let e = {
                        let l3 = *ptr1.add(4).cast::<*mut u8>();
                        let l4 = *ptr1.add(8).cast::<usize>();
                        let len5 = l4;
                        let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                        _rt::string_lift(bytes5)
                    };
                    Some(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl SharedContext {
    #[allow(unused_unsafe, clippy::all)]
    /// Fetches a context value with the given name, if existing.
    pub fn get(&self, name: &str) -> Option<_rt::String> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]shared-context.get"]
                fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
            let l2 = i32::from(*ptr1.add(0).cast::<u8>());
            match l2 {
                0 => None,
                1 => {
                    let e = {
                        let l3 = *ptr1.add(4).cast::<*mut u8>();
                        let l4 = *ptr1.add(8).cast::<usize>();
                        let len5 = l4;
                        let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                        _rt::string_lift(bytes5)
                    };
                    Some(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl SharedContext {
    #[allow(unused_unsafe, clippy::all)]
    /// Gets the current trace-id.
    pub fn trace_id(&self) -> _rt::String {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
            let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]shared-context.trace-id"]
                fn wit_import(_: i32, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0);
            let l1 = *ptr0.add(0).cast::<*mut u8>();
            let l2 = *ptr0.add(4).cast::<usize>();
            let len3 = l2;
            let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
            _rt::string_lift(bytes3)
        }
    }
}
impl Headers {
    #[allow(unused_unsafe, clippy::all)]
    /// Gets a header value with the given name.
    pub fn get(&self, name: &str) -> Option<_rt::String> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]headers.get"]
                fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
            let l2 = i32::from(*ptr1.add(0).cast::<u8>());
            match l2 {
                0 => None,
                1 => {
                    let e = {
                        let l3 = *ptr1.add(4).cast::<*mut u8>();
                        let l4 = *ptr1.add(8).cast::<usize>();
                        let len5 = l4;
                        let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                        _rt::string_lift(bytes5)
                    };
                    Some(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl Headers {
    #[allow(unused_unsafe, clippy::all)]
    /// Sets the header value with the given name. Returns an error if the given name
    /// is not a valid header name.
    pub fn set(&self, name: &str, value: &str) -> Result<(), HeaderError> {
        unsafe {
            #[repr(align(1))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 2]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 2]);
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let vec1 = value;
            let ptr1 = vec1.as_ptr().cast::<u8>();
            let len1 = vec1.len();
            let ptr2 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]headers.set"]
                fn wit_import(
                    _: i32,
                    _: *mut u8,
                    _: usize,
                    _: *mut u8,
                    _: usize,
                    _: *mut u8,
                );
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(
                _: i32,
                _: *mut u8,
                _: usize,
                _: *mut u8,
                _: usize,
                _: *mut u8,
            ) {
                unreachable!()
            }
            wit_import(
                (self).handle() as i32,
                ptr0.cast_mut(),
                len0,
                ptr1.cast_mut(),
                len1,
                ptr2,
            );
            let l3 = i32::from(*ptr2.add(0).cast::<u8>());
            match l3 {
                0 => {
                    let e = ();
                    Ok(e)
                }
                1 => {
                    let e = {
                        let l4 = i32::from(*ptr2.add(1).cast::<u8>());
                        HeaderError::_lift(l4 as u8)
                    };
                    Err(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl Headers {
    #[allow(unused_unsafe, clippy::all)]
    /// Deletes a header value with the given name.
    pub fn delete(&self, name: &str) -> Option<_rt::String> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
            let vec0 = name;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]headers.delete"]
                fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
            let l2 = i32::from(*ptr1.add(0).cast::<u8>());
            match l2 {
                0 => None,
                1 => {
                    let e = {
                        let l3 = *ptr1.add(4).cast::<*mut u8>();
                        let l4 = *ptr1.add(8).cast::<usize>();
                        let len5 = l4;
                        let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                        _rt::string_lift(bytes5)
                    };
                    Some(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl Headers {
    #[allow(unused_unsafe, clippy::all)]
    /// Return all headers as a list of tuples.
    pub fn entries(&self) -> _rt::Vec<(_rt::String, _rt::String)> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
            let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[method]headers.entries"]
                fn wit_import(_: i32, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: i32, _: *mut u8) {
                unreachable!()
            }
            wit_import((self).handle() as i32, ptr0);
            let l1 = *ptr0.add(0).cast::<*mut u8>();
            let l2 = *ptr0.add(4).cast::<usize>();
            let base9 = l1;
            let len9 = l2;
            let mut result9 = _rt::Vec::with_capacity(len9);
            for i in 0..len9 {
                let base = base9.add(i * 16);
                let e9 = {
                    let l3 = *base.add(0).cast::<*mut u8>();
                    let l4 = *base.add(4).cast::<usize>();
                    let len5 = l4;
                    let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                    let l6 = *base.add(8).cast::<*mut u8>();
                    let l7 = *base.add(12).cast::<usize>();
                    let len8 = l7;
                    let bytes8 = _rt::Vec::from_raw_parts(l6.cast(), len8, len8);
                    (_rt::string_lift(bytes5), _rt::string_lift(bytes8))
                };
                result9.push(e9);
            }
            _rt::cabi_dealloc(base9, len9 * 16, 4);
            result9
        }
    }
}
impl HttpClient {
    #[allow(unused_unsafe, clippy::all)]
    /// Executes a request and returns the response, yielding the current future until finished.
    pub fn execute(request: &HttpRequest) -> Result<HttpResponse, HttpError> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 24]);
            let HttpRequest {
                method: method0,
                url: url0,
                headers: headers0,
                body: body0,
                timeout_ms: timeout_ms0,
            } = request;
            let vec1 = url0;
            let ptr1 = vec1.as_ptr().cast::<u8>();
            let len1 = vec1.len();
            let vec5 = headers0;
            let len5 = vec5.len();
            let layout5 = _rt::alloc::Layout::from_size_align_unchecked(
                vec5.len() * 16,
                4,
            );
            let result5 = if layout5.size() != 0 {
                let ptr = _rt::alloc::alloc(layout5).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout5);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec5.into_iter().enumerate() {
                let base = result5.add(i * 16);
                {
                    let (t2_0, t2_1) = e;
                    let vec3 = t2_0;
                    let ptr3 = vec3.as_ptr().cast::<u8>();
                    let len3 = vec3.len();
                    *base.add(4).cast::<usize>() = len3;
                    *base.add(0).cast::<*mut u8>() = ptr3.cast_mut();
                    let vec4 = t2_1;
                    let ptr4 = vec4.as_ptr().cast::<u8>();
                    let len4 = vec4.len();
                    *base.add(12).cast::<usize>() = len4;
                    *base.add(8).cast::<*mut u8>() = ptr4.cast_mut();
                }
            }
            let vec6 = body0;
            let ptr6 = vec6.as_ptr().cast::<u8>();
            let len6 = vec6.len();
            let (result7_0, result7_1) = match timeout_ms0 {
                Some(e) => (1i32, _rt::as_i64(e)),
                None => (0i32, 0i64),
            };
            let ptr8 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[static]http-client.execute"]
                fn wit_import(
                    _: i32,
                    _: *mut u8,
                    _: usize,
                    _: *mut u8,
                    _: usize,
                    _: *mut u8,
                    _: usize,
                    _: i32,
                    _: i64,
                    _: *mut u8,
                );
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(
                _: i32,
                _: *mut u8,
                _: usize,
                _: *mut u8,
                _: usize,
                _: *mut u8,
                _: usize,
                _: i32,
                _: i64,
                _: *mut u8,
            ) {
                unreachable!()
            }
            wit_import(
                method0.clone() as i32,
                ptr1.cast_mut(),
                len1,
                result5,
                len5,
                ptr6.cast_mut(),
                len6,
                result7_0,
                result7_1,
                ptr8,
            );
            let l9 = i32::from(*ptr8.add(0).cast::<u8>());
            if layout5.size() != 0 {
                _rt::alloc::dealloc(result5.cast(), layout5);
            }
            match l9 {
                0 => {
                    let e = {
                        let l10 = i32::from(*ptr8.add(4).cast::<u16>());
                        let l11 = i32::from(*ptr8.add(6).cast::<u8>());
                        let l12 = *ptr8.add(8).cast::<*mut u8>();
                        let l13 = *ptr8.add(12).cast::<usize>();
                        let base20 = l12;
                        let len20 = l13;
                        let mut result20 = _rt::Vec::with_capacity(len20);
                        for i in 0..len20 {
                            let base = base20.add(i * 16);
                            let e20 = {
                                let l14 = *base.add(0).cast::<*mut u8>();
                                let l15 = *base.add(4).cast::<usize>();
                                let len16 = l15;
                                let bytes16 = _rt::Vec::from_raw_parts(
                                    l14.cast(),
                                    len16,
                                    len16,
                                );
                                let l17 = *base.add(8).cast::<*mut u8>();
                                let l18 = *base.add(12).cast::<usize>();
                                let len19 = l18;
                                let bytes19 = _rt::Vec::from_raw_parts(
                                    l17.cast(),
                                    len19,
                                    len19,
                                );
                                (_rt::string_lift(bytes16), _rt::string_lift(bytes19))
                            };
                            result20.push(e20);
                        }
                        _rt::cabi_dealloc(base20, len20 * 16, 4);
                        let l21 = *ptr8.add(16).cast::<*mut u8>();
                        let l22 = *ptr8.add(20).cast::<usize>();
                        let len23 = l22;
                        HttpResponse {
                            status: l10 as u16,
                            version: HttpVersion::_lift(l11 as u8),
                            headers: result20,
                            body: _rt::Vec::from_raw_parts(l21.cast(), len23, len23),
                        }
                    };
                    Ok(e)
                }
                1 => {
                    let e = {
                        let l24 = i32::from(*ptr8.add(4).cast::<u8>());
                        let v31 = match l24 {
                            0 => HttpError::Timeout,
                            1 => {
                                let e31 = {
                                    let l25 = *ptr8.add(8).cast::<*mut u8>();
                                    let l26 = *ptr8.add(12).cast::<usize>();
                                    let len27 = l26;
                                    let bytes27 = _rt::Vec::from_raw_parts(
                                        l25.cast(),
                                        len27,
                                        len27,
                                    );
                                    _rt::string_lift(bytes27)
                                };
                                HttpError::Request(e31)
                            }
                            n => {
                                debug_assert_eq!(n, 2, "invalid enum discriminant");
                                let e31 = {
                                    let l28 = *ptr8.add(8).cast::<*mut u8>();
                                    let l29 = *ptr8.add(12).cast::<usize>();
                                    let len30 = l29;
                                    let bytes30 = _rt::Vec::from_raw_parts(
                                        l28.cast(),
                                        len30,
                                        len30,
                                    );
                                    _rt::string_lift(bytes30)
                                };
                                HttpError::Connect(e31)
                            }
                        };
                        v31
                    };
                    Err(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
impl HttpClient {
    #[allow(unused_unsafe, clippy::all)]
    /// Executes multiple requests in parallel, yielding the current future until all requests are done.
    pub fn execute_many(
        requests: &[HttpRequest],
    ) -> _rt::Vec<Result<HttpResponse, HttpError>> {
        unsafe {
            let mut cleanup_list = _rt::Vec::new();
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
            let vec7 = requests;
            let len7 = vec7.len();
            let layout7 = _rt::alloc::Layout::from_size_align_unchecked(
                vec7.len() * 48,
                8,
            );
            let result7 = if layout7.size() != 0 {
                let ptr = _rt::alloc::alloc(layout7).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout7);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec7.into_iter().enumerate() {
                let base = result7.add(i * 48);
                {
                    let HttpRequest {
                        method: method0,
                        url: url0,
                        headers: headers0,
                        body: body0,
                        timeout_ms: timeout_ms0,
                    } = e;
                    *base.add(0).cast::<u8>() = (method0.clone() as i32) as u8;
                    let vec1 = url0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    *base.add(8).cast::<usize>() = len1;
                    *base.add(4).cast::<*mut u8>() = ptr1.cast_mut();
                    let vec5 = headers0;
                    let len5 = vec5.len();
                    let layout5 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec5.len() * 16,
                        4,
                    );
                    let result5 = if layout5.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout5).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout5);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec5.into_iter().enumerate() {
                        let base = result5.add(i * 16);
                        {
                            let (t2_0, t2_1) = e;
                            let vec3 = t2_0;
                            let ptr3 = vec3.as_ptr().cast::<u8>();
                            let len3 = vec3.len();
                            *base.add(4).cast::<usize>() = len3;
                            *base.add(0).cast::<*mut u8>() = ptr3.cast_mut();
                            let vec4 = t2_1;
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            *base.add(12).cast::<usize>() = len4;
                            *base.add(8).cast::<*mut u8>() = ptr4.cast_mut();
                        }
                    }
                    *base.add(16).cast::<usize>() = len5;
                    *base.add(12).cast::<*mut u8>() = result5;
                    let vec6 = body0;
                    let ptr6 = vec6.as_ptr().cast::<u8>();
                    let len6 = vec6.len();
                    *base.add(24).cast::<usize>() = len6;
                    *base.add(20).cast::<*mut u8>() = ptr6.cast_mut();
                    match timeout_ms0 {
                        Some(e) => {
                            *base.add(32).cast::<u8>() = (1i32) as u8;
                            *base.add(40).cast::<i64>() = _rt::as_i64(e);
                        }
                        None => {
                            *base.add(32).cast::<u8>() = (0i32) as u8;
                        }
                    };
                    cleanup_list.extend_from_slice(&[(result5, layout5)]);
                }
            }
            let ptr8 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[static]http-client.execute-many"]
                fn wit_import(_: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import(result7, len7, ptr8);
            let l9 = *ptr8.add(0).cast::<*mut u8>();
            let l10 = *ptr8.add(4).cast::<usize>();
            let base34 = l9;
            let len34 = l10;
            let mut result34 = _rt::Vec::with_capacity(len34);
            for i in 0..len34 {
                let base = base34.add(i * 24);
                let e34 = {
                    let l11 = i32::from(*base.add(0).cast::<u8>());
                    match l11 {
                        0 => {
                            let e = {
                                let l12 = i32::from(*base.add(4).cast::<u16>());
                                let l13 = i32::from(*base.add(6).cast::<u8>());
                                let l14 = *base.add(8).cast::<*mut u8>();
                                let l15 = *base.add(12).cast::<usize>();
                                let base22 = l14;
                                let len22 = l15;
                                let mut result22 = _rt::Vec::with_capacity(len22);
                                for i in 0..len22 {
                                    let base = base22.add(i * 16);
                                    let e22 = {
                                        let l16 = *base.add(0).cast::<*mut u8>();
                                        let l17 = *base.add(4).cast::<usize>();
                                        let len18 = l17;
                                        let bytes18 = _rt::Vec::from_raw_parts(
                                            l16.cast(),
                                            len18,
                                            len18,
                                        );
                                        let l19 = *base.add(8).cast::<*mut u8>();
                                        let l20 = *base.add(12).cast::<usize>();
                                        let len21 = l20;
                                        let bytes21 = _rt::Vec::from_raw_parts(
                                            l19.cast(),
                                            len21,
                                            len21,
                                        );
                                        (_rt::string_lift(bytes18), _rt::string_lift(bytes21))
                                    };
                                    result22.push(e22);
                                }
                                _rt::cabi_dealloc(base22, len22 * 16, 4);
                                let l23 = *base.add(16).cast::<*mut u8>();
                                let l24 = *base.add(20).cast::<usize>();
                                let len25 = l24;
                                HttpResponse {
                                    status: l12 as u16,
                                    version: HttpVersion::_lift(l13 as u8),
                                    headers: result22,
                                    body: _rt::Vec::from_raw_parts(l23.cast(), len25, len25),
                                }
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l26 = i32::from(*base.add(4).cast::<u8>());
                                let v33 = match l26 {
                                    0 => HttpError::Timeout,
                                    1 => {
                                        let e33 = {
                                            let l27 = *base.add(8).cast::<*mut u8>();
                                            let l28 = *base.add(12).cast::<usize>();
                                            let len29 = l28;
                                            let bytes29 = _rt::Vec::from_raw_parts(
                                                l27.cast(),
                                                len29,
                                                len29,
                                            );
                                            _rt::string_lift(bytes29)
                                        };
                                        HttpError::Request(e33)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 2, "invalid enum discriminant");
                                        let e33 = {
                                            let l30 = *base.add(8).cast::<*mut u8>();
                                            let l31 = *base.add(12).cast::<usize>();
                                            let len32 = l31;
                                            let bytes32 = _rt::Vec::from_raw_parts(
                                                l30.cast(),
                                                len32,
                                                len32,
                                            );
                                            _rt::string_lift(bytes32)
                                        };
                                        HttpError::Connect(e33)
                                    }
                                };
                                v33
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                };
                result34.push(e34);
            }
            _rt::cabi_dealloc(base34, len34 * 24, 4);
            if layout7.size() != 0 {
                _rt::alloc::dealloc(result7.cast(), layout7);
            }
            for (ptr, layout) in cleanup_list {
                if layout.size() != 0 {
                    _rt::alloc::dealloc(ptr.cast(), layout);
                }
            }
            result34
        }
    }
}
impl AccessLog {
    #[allow(unused_unsafe, clippy::all)]
    /// Sends the data to the access log.
    pub fn send(data: &[u8]) -> Result<(), LogError> {
        unsafe {
            #[repr(align(4))]
            struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
            let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
            let vec0 = data;
            let ptr0 = vec0.as_ptr().cast::<u8>();
            let len0 = vec0.len();
            let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "wasm32")]
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[static]access-log.send"]
                fn wit_import(_: *mut u8, _: usize, _: *mut u8);
            }
            #[cfg(not(target_arch = "wasm32"))]
            fn wit_import(_: *mut u8, _: usize, _: *mut u8) {
                unreachable!()
            }
            wit_import(ptr0.cast_mut(), len0, ptr1);
            let l2 = i32::from(*ptr1.add(0).cast::<u8>());
            match l2 {
                0 => {
                    let e = ();
                    Ok(e)
                }
                1 => {
                    let e = {
                        let l3 = i32::from(*ptr1.add(4).cast::<u8>());
                        let v7 = match l3 {
                            0 => {
                                let e7 = {
                                    let l4 = *ptr1.add(8).cast::<*mut u8>();
                                    let l5 = *ptr1.add(12).cast::<usize>();
                                    let len6 = l5;
                                    _rt::Vec::from_raw_parts(l4.cast(), len6, len6)
                                };
                                LogError::ChannelFull(e7)
                            }
                            n => {
                                debug_assert_eq!(n, 1, "invalid enum discriminant");
                                LogError::ChannelClosed
                            }
                        };
                        v7
                    };
                    Err(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            }
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_on_gateway_request_cabi<T: Guest>(
    arg0: i32,
    arg1: i32,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let result0 = T::on_gateway_request(
        Context::from_handle(arg0 as u32),
        Headers::from_handle(arg1 as u32),
    );
    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result0 {
        Ok(_) => {
            *ptr1.add(0).cast::<u8>() = (0i32) as u8;
        }
        Err(e) => {
            *ptr1.add(0).cast::<u8>() = (1i32) as u8;
            let ErrorResponse { status_code: status_code2, errors: errors2 } = e;
            *ptr1.add(4).cast::<u16>() = (_rt::as_i32(status_code2)) as u16;
            let vec9 = errors2;
            let len9 = vec9.len();
            let layout9 = _rt::alloc::Layout::from_size_align_unchecked(
                vec9.len() * 16,
                4,
            );
            let result9 = if layout9.size() != 0 {
                let ptr = _rt::alloc::alloc(layout9).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout9);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec9.into_iter().enumerate() {
                let base = result9.add(i * 16);
                {
                    let Error { extensions: extensions3, message: message3 } = e;
                    let vec7 = extensions3;
                    let len7 = vec7.len();
                    let layout7 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec7.len() * 16,
                        4,
                    );
                    let result7 = if layout7.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout7).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout7);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec7.into_iter().enumerate() {
                        let base = result7.add(i * 16);
                        {
                            let (t4_0, t4_1) = e;
                            let vec5 = (t4_0.into_bytes()).into_boxed_slice();
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            ::core::mem::forget(vec5);
                            *base.add(4).cast::<usize>() = len5;
                            *base.add(0).cast::<*mut u8>() = ptr5.cast_mut();
                            let vec6 = (t4_1.into_bytes()).into_boxed_slice();
                            let ptr6 = vec6.as_ptr().cast::<u8>();
                            let len6 = vec6.len();
                            ::core::mem::forget(vec6);
                            *base.add(12).cast::<usize>() = len6;
                            *base.add(8).cast::<*mut u8>() = ptr6.cast_mut();
                        }
                    }
                    *base.add(4).cast::<usize>() = len7;
                    *base.add(0).cast::<*mut u8>() = result7;
                    let vec8 = (message3.into_bytes()).into_boxed_slice();
                    let ptr8 = vec8.as_ptr().cast::<u8>();
                    let len8 = vec8.len();
                    ::core::mem::forget(vec8);
                    *base.add(12).cast::<usize>() = len8;
                    *base.add(8).cast::<*mut u8>() = ptr8.cast_mut();
                }
            }
            *ptr1.add(12).cast::<usize>() = len9;
            *ptr1.add(8).cast::<*mut u8>() = result9;
        }
    };
    ptr1
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_on_gateway_request<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {}
        _ => {
            let l1 = *arg0.add(8).cast::<*mut u8>();
            let l2 = *arg0.add(12).cast::<usize>();
            let base12 = l1;
            let len12 = l2;
            for i in 0..len12 {
                let base = base12.add(i * 16);
                {
                    let l3 = *base.add(0).cast::<*mut u8>();
                    let l4 = *base.add(4).cast::<usize>();
                    let base9 = l3;
                    let len9 = l4;
                    for i in 0..len9 {
                        let base = base9.add(i * 16);
                        {
                            let l5 = *base.add(0).cast::<*mut u8>();
                            let l6 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l5, l6, 1);
                            let l7 = *base.add(8).cast::<*mut u8>();
                            let l8 = *base.add(12).cast::<usize>();
                            _rt::cabi_dealloc(l7, l8, 1);
                        }
                    }
                    _rt::cabi_dealloc(base9, len9 * 16, 4);
                    let l10 = *base.add(8).cast::<*mut u8>();
                    let l11 = *base.add(12).cast::<usize>();
                    _rt::cabi_dealloc(l10, l11, 1);
                }
            }
            _rt::cabi_dealloc(base12, len12 * 16, 4);
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_on_subgraph_request_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: i32,
    arg4: *mut u8,
    arg5: usize,
    arg6: i32,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg5;
    let bytes1 = _rt::Vec::from_raw_parts(arg4.cast(), len1, len1);
    let result2 = T::on_subgraph_request(
        SharedContext::from_handle(arg0 as u32),
        _rt::string_lift(bytes0),
        HttpMethod::_lift(arg3 as u8),
        _rt::string_lift(bytes1),
        Headers::from_handle(arg6 as u32),
    );
    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result2 {
        Ok(_) => {
            *ptr3.add(0).cast::<u8>() = (0i32) as u8;
        }
        Err(e) => {
            *ptr3.add(0).cast::<u8>() = (1i32) as u8;
            let Error { extensions: extensions4, message: message4 } = e;
            let vec8 = extensions4;
            let len8 = vec8.len();
            let layout8 = _rt::alloc::Layout::from_size_align_unchecked(
                vec8.len() * 16,
                4,
            );
            let result8 = if layout8.size() != 0 {
                let ptr = _rt::alloc::alloc(layout8).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout8);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec8.into_iter().enumerate() {
                let base = result8.add(i * 16);
                {
                    let (t5_0, t5_1) = e;
                    let vec6 = (t5_0.into_bytes()).into_boxed_slice();
                    let ptr6 = vec6.as_ptr().cast::<u8>();
                    let len6 = vec6.len();
                    ::core::mem::forget(vec6);
                    *base.add(4).cast::<usize>() = len6;
                    *base.add(0).cast::<*mut u8>() = ptr6.cast_mut();
                    let vec7 = (t5_1.into_bytes()).into_boxed_slice();
                    let ptr7 = vec7.as_ptr().cast::<u8>();
                    let len7 = vec7.len();
                    ::core::mem::forget(vec7);
                    *base.add(12).cast::<usize>() = len7;
                    *base.add(8).cast::<*mut u8>() = ptr7.cast_mut();
                }
            }
            *ptr3.add(8).cast::<usize>() = len8;
            *ptr3.add(4).cast::<*mut u8>() = result8;
            let vec9 = (message4.into_bytes()).into_boxed_slice();
            let ptr9 = vec9.as_ptr().cast::<u8>();
            let len9 = vec9.len();
            ::core::mem::forget(vec9);
            *ptr3.add(16).cast::<usize>() = len9;
            *ptr3.add(12).cast::<*mut u8>() = ptr9.cast_mut();
        }
    };
    ptr3
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_on_subgraph_request<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {}
        _ => {
            let l1 = *arg0.add(4).cast::<*mut u8>();
            let l2 = *arg0.add(8).cast::<usize>();
            let base7 = l1;
            let len7 = l2;
            for i in 0..len7 {
                let base = base7.add(i * 16);
                {
                    let l3 = *base.add(0).cast::<*mut u8>();
                    let l4 = *base.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l3, l4, 1);
                    let l5 = *base.add(8).cast::<*mut u8>();
                    let l6 = *base.add(12).cast::<usize>();
                    _rt::cabi_dealloc(l5, l6, 1);
                }
            }
            _rt::cabi_dealloc(base7, len7 * 16, 4);
            let l8 = *arg0.add(12).cast::<*mut u8>();
            let l9 = *arg0.add(16).cast::<usize>();
            _rt::cabi_dealloc(l8, l9, 1);
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_authorize_edge_pre_execution_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
    arg5: *mut u8,
    arg6: usize,
    arg7: *mut u8,
    arg8: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let len2 = arg6;
    let bytes2 = _rt::Vec::from_raw_parts(arg5.cast(), len2, len2);
    let len3 = arg8;
    let bytes3 = _rt::Vec::from_raw_parts(arg7.cast(), len3, len3);
    let result4 = T::authorize_edge_pre_execution(
        SharedContext::from_handle(arg0 as u32),
        EdgeDefinition {
            parent_type_name: _rt::string_lift(bytes0),
            field_name: _rt::string_lift(bytes1),
        },
        _rt::string_lift(bytes2),
        _rt::string_lift(bytes3),
    );
    let ptr5 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result4 {
        Ok(_) => {
            *ptr5.add(0).cast::<u8>() = (0i32) as u8;
        }
        Err(e) => {
            *ptr5.add(0).cast::<u8>() = (1i32) as u8;
            let Error { extensions: extensions6, message: message6 } = e;
            let vec10 = extensions6;
            let len10 = vec10.len();
            let layout10 = _rt::alloc::Layout::from_size_align_unchecked(
                vec10.len() * 16,
                4,
            );
            let result10 = if layout10.size() != 0 {
                let ptr = _rt::alloc::alloc(layout10).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout10);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec10.into_iter().enumerate() {
                let base = result10.add(i * 16);
                {
                    let (t7_0, t7_1) = e;
                    let vec8 = (t7_0.into_bytes()).into_boxed_slice();
                    let ptr8 = vec8.as_ptr().cast::<u8>();
                    let len8 = vec8.len();
                    ::core::mem::forget(vec8);
                    *base.add(4).cast::<usize>() = len8;
                    *base.add(0).cast::<*mut u8>() = ptr8.cast_mut();
                    let vec9 = (t7_1.into_bytes()).into_boxed_slice();
                    let ptr9 = vec9.as_ptr().cast::<u8>();
                    let len9 = vec9.len();
                    ::core::mem::forget(vec9);
                    *base.add(12).cast::<usize>() = len9;
                    *base.add(8).cast::<*mut u8>() = ptr9.cast_mut();
                }
            }
            *ptr5.add(8).cast::<usize>() = len10;
            *ptr5.add(4).cast::<*mut u8>() = result10;
            let vec11 = (message6.into_bytes()).into_boxed_slice();
            let ptr11 = vec11.as_ptr().cast::<u8>();
            let len11 = vec11.len();
            ::core::mem::forget(vec11);
            *ptr5.add(16).cast::<usize>() = len11;
            *ptr5.add(12).cast::<*mut u8>() = ptr11.cast_mut();
        }
    };
    ptr5
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_authorize_edge_pre_execution<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {}
        _ => {
            let l1 = *arg0.add(4).cast::<*mut u8>();
            let l2 = *arg0.add(8).cast::<usize>();
            let base7 = l1;
            let len7 = l2;
            for i in 0..len7 {
                let base = base7.add(i * 16);
                {
                    let l3 = *base.add(0).cast::<*mut u8>();
                    let l4 = *base.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l3, l4, 1);
                    let l5 = *base.add(8).cast::<*mut u8>();
                    let l6 = *base.add(12).cast::<usize>();
                    _rt::cabi_dealloc(l5, l6, 1);
                }
            }
            _rt::cabi_dealloc(base7, len7 * 16, 4);
            let l8 = *arg0.add(12).cast::<*mut u8>();
            let l9 = *arg0.add(16).cast::<usize>();
            _rt::cabi_dealloc(l8, l9, 1);
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_authorize_node_pre_execution_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let result2 = T::authorize_node_pre_execution(
        SharedContext::from_handle(arg0 as u32),
        NodeDefinition {
            type_name: _rt::string_lift(bytes0),
        },
        _rt::string_lift(bytes1),
    );
    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result2 {
        Ok(_) => {
            *ptr3.add(0).cast::<u8>() = (0i32) as u8;
        }
        Err(e) => {
            *ptr3.add(0).cast::<u8>() = (1i32) as u8;
            let Error { extensions: extensions4, message: message4 } = e;
            let vec8 = extensions4;
            let len8 = vec8.len();
            let layout8 = _rt::alloc::Layout::from_size_align_unchecked(
                vec8.len() * 16,
                4,
            );
            let result8 = if layout8.size() != 0 {
                let ptr = _rt::alloc::alloc(layout8).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout8);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec8.into_iter().enumerate() {
                let base = result8.add(i * 16);
                {
                    let (t5_0, t5_1) = e;
                    let vec6 = (t5_0.into_bytes()).into_boxed_slice();
                    let ptr6 = vec6.as_ptr().cast::<u8>();
                    let len6 = vec6.len();
                    ::core::mem::forget(vec6);
                    *base.add(4).cast::<usize>() = len6;
                    *base.add(0).cast::<*mut u8>() = ptr6.cast_mut();
                    let vec7 = (t5_1.into_bytes()).into_boxed_slice();
                    let ptr7 = vec7.as_ptr().cast::<u8>();
                    let len7 = vec7.len();
                    ::core::mem::forget(vec7);
                    *base.add(12).cast::<usize>() = len7;
                    *base.add(8).cast::<*mut u8>() = ptr7.cast_mut();
                }
            }
            *ptr3.add(8).cast::<usize>() = len8;
            *ptr3.add(4).cast::<*mut u8>() = result8;
            let vec9 = (message4.into_bytes()).into_boxed_slice();
            let ptr9 = vec9.as_ptr().cast::<u8>();
            let len9 = vec9.len();
            ::core::mem::forget(vec9);
            *ptr3.add(16).cast::<usize>() = len9;
            *ptr3.add(12).cast::<*mut u8>() = ptr9.cast_mut();
        }
    };
    ptr3
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_authorize_node_pre_execution<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {}
        _ => {
            let l1 = *arg0.add(4).cast::<*mut u8>();
            let l2 = *arg0.add(8).cast::<usize>();
            let base7 = l1;
            let len7 = l2;
            for i in 0..len7 {
                let base = base7.add(i * 16);
                {
                    let l3 = *base.add(0).cast::<*mut u8>();
                    let l4 = *base.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l3, l4, 1);
                    let l5 = *base.add(8).cast::<*mut u8>();
                    let l6 = *base.add(12).cast::<usize>();
                    _rt::cabi_dealloc(l5, l6, 1);
                }
            }
            _rt::cabi_dealloc(base7, len7 * 16, 4);
            let l8 = *arg0.add(12).cast::<*mut u8>();
            let l9 = *arg0.add(16).cast::<usize>();
            _rt::cabi_dealloc(l8, l9, 1);
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_authorize_parent_edge_post_execution_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
    arg5: *mut u8,
    arg6: usize,
    arg7: *mut u8,
    arg8: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let base5 = arg5;
    let len5 = arg6;
    let mut result5 = _rt::Vec::with_capacity(len5);
    for i in 0..len5 {
        let base = base5.add(i * 8);
        let e5 = {
            let l2 = *base.add(0).cast::<*mut u8>();
            let l3 = *base.add(4).cast::<usize>();
            let len4 = l3;
            let bytes4 = _rt::Vec::from_raw_parts(l2.cast(), len4, len4);
            _rt::string_lift(bytes4)
        };
        result5.push(e5);
    }
    _rt::cabi_dealloc(base5, len5 * 8, 4);
    let len6 = arg8;
    let bytes6 = _rt::Vec::from_raw_parts(arg7.cast(), len6, len6);
    let result7 = T::authorize_parent_edge_post_execution(
        SharedContext::from_handle(arg0 as u32),
        EdgeDefinition {
            parent_type_name: _rt::string_lift(bytes0),
            field_name: _rt::string_lift(bytes1),
        },
        result5,
        _rt::string_lift(bytes6),
    );
    let ptr8 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    let vec15 = result7;
    let len15 = vec15.len();
    let layout15 = _rt::alloc::Layout::from_size_align_unchecked(vec15.len() * 20, 4);
    let result15 = if layout15.size() != 0 {
        let ptr = _rt::alloc::alloc(layout15).cast::<u8>();
        if ptr.is_null() {
            _rt::alloc::handle_alloc_error(layout15);
        }
        ptr
    } else {
        ::core::ptr::null_mut()
    };
    for (i, e) in vec15.into_iter().enumerate() {
        let base = result15.add(i * 20);
        {
            match e {
                Ok(_) => {
                    *base.add(0).cast::<u8>() = (0i32) as u8;
                }
                Err(e) => {
                    *base.add(0).cast::<u8>() = (1i32) as u8;
                    let Error { extensions: extensions9, message: message9 } = e;
                    let vec13 = extensions9;
                    let len13 = vec13.len();
                    let layout13 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec13.len() * 16,
                        4,
                    );
                    let result13 = if layout13.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout13).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout13);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec13.into_iter().enumerate() {
                        let base = result13.add(i * 16);
                        {
                            let (t10_0, t10_1) = e;
                            let vec11 = (t10_0.into_bytes()).into_boxed_slice();
                            let ptr11 = vec11.as_ptr().cast::<u8>();
                            let len11 = vec11.len();
                            ::core::mem::forget(vec11);
                            *base.add(4).cast::<usize>() = len11;
                            *base.add(0).cast::<*mut u8>() = ptr11.cast_mut();
                            let vec12 = (t10_1.into_bytes()).into_boxed_slice();
                            let ptr12 = vec12.as_ptr().cast::<u8>();
                            let len12 = vec12.len();
                            ::core::mem::forget(vec12);
                            *base.add(12).cast::<usize>() = len12;
                            *base.add(8).cast::<*mut u8>() = ptr12.cast_mut();
                        }
                    }
                    *base.add(8).cast::<usize>() = len13;
                    *base.add(4).cast::<*mut u8>() = result13;
                    let vec14 = (message9.into_bytes()).into_boxed_slice();
                    let ptr14 = vec14.as_ptr().cast::<u8>();
                    let len14 = vec14.len();
                    ::core::mem::forget(vec14);
                    *base.add(16).cast::<usize>() = len14;
                    *base.add(12).cast::<*mut u8>() = ptr14.cast_mut();
                }
            };
        }
    }
    *ptr8.add(4).cast::<usize>() = len15;
    *ptr8.add(0).cast::<*mut u8>() = result15;
    ptr8
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_authorize_parent_edge_post_execution<T: Guest>(
    arg0: *mut u8,
) {
    let l0 = *arg0.add(0).cast::<*mut u8>();
    let l1 = *arg0.add(4).cast::<usize>();
    let base12 = l0;
    let len12 = l1;
    for i in 0..len12 {
        let base = base12.add(i * 20);
        {
            let l2 = i32::from(*base.add(0).cast::<u8>());
            match l2 {
                0 => {}
                _ => {
                    let l3 = *base.add(4).cast::<*mut u8>();
                    let l4 = *base.add(8).cast::<usize>();
                    let base9 = l3;
                    let len9 = l4;
                    for i in 0..len9 {
                        let base = base9.add(i * 16);
                        {
                            let l5 = *base.add(0).cast::<*mut u8>();
                            let l6 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l5, l6, 1);
                            let l7 = *base.add(8).cast::<*mut u8>();
                            let l8 = *base.add(12).cast::<usize>();
                            _rt::cabi_dealloc(l7, l8, 1);
                        }
                    }
                    _rt::cabi_dealloc(base9, len9 * 16, 4);
                    let l10 = *base.add(12).cast::<*mut u8>();
                    let l11 = *base.add(16).cast::<usize>();
                    _rt::cabi_dealloc(l10, l11, 1);
                }
            }
        }
    }
    _rt::cabi_dealloc(base12, len12 * 20, 4);
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_authorize_edge_node_post_execution_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
    arg5: *mut u8,
    arg6: usize,
    arg7: *mut u8,
    arg8: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let base5 = arg5;
    let len5 = arg6;
    let mut result5 = _rt::Vec::with_capacity(len5);
    for i in 0..len5 {
        let base = base5.add(i * 8);
        let e5 = {
            let l2 = *base.add(0).cast::<*mut u8>();
            let l3 = *base.add(4).cast::<usize>();
            let len4 = l3;
            let bytes4 = _rt::Vec::from_raw_parts(l2.cast(), len4, len4);
            _rt::string_lift(bytes4)
        };
        result5.push(e5);
    }
    _rt::cabi_dealloc(base5, len5 * 8, 4);
    let len6 = arg8;
    let bytes6 = _rt::Vec::from_raw_parts(arg7.cast(), len6, len6);
    let result7 = T::authorize_edge_node_post_execution(
        SharedContext::from_handle(arg0 as u32),
        EdgeDefinition {
            parent_type_name: _rt::string_lift(bytes0),
            field_name: _rt::string_lift(bytes1),
        },
        result5,
        _rt::string_lift(bytes6),
    );
    let ptr8 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    let vec15 = result7;
    let len15 = vec15.len();
    let layout15 = _rt::alloc::Layout::from_size_align_unchecked(vec15.len() * 20, 4);
    let result15 = if layout15.size() != 0 {
        let ptr = _rt::alloc::alloc(layout15).cast::<u8>();
        if ptr.is_null() {
            _rt::alloc::handle_alloc_error(layout15);
        }
        ptr
    } else {
        ::core::ptr::null_mut()
    };
    for (i, e) in vec15.into_iter().enumerate() {
        let base = result15.add(i * 20);
        {
            match e {
                Ok(_) => {
                    *base.add(0).cast::<u8>() = (0i32) as u8;
                }
                Err(e) => {
                    *base.add(0).cast::<u8>() = (1i32) as u8;
                    let Error { extensions: extensions9, message: message9 } = e;
                    let vec13 = extensions9;
                    let len13 = vec13.len();
                    let layout13 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec13.len() * 16,
                        4,
                    );
                    let result13 = if layout13.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout13).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout13);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec13.into_iter().enumerate() {
                        let base = result13.add(i * 16);
                        {
                            let (t10_0, t10_1) = e;
                            let vec11 = (t10_0.into_bytes()).into_boxed_slice();
                            let ptr11 = vec11.as_ptr().cast::<u8>();
                            let len11 = vec11.len();
                            ::core::mem::forget(vec11);
                            *base.add(4).cast::<usize>() = len11;
                            *base.add(0).cast::<*mut u8>() = ptr11.cast_mut();
                            let vec12 = (t10_1.into_bytes()).into_boxed_slice();
                            let ptr12 = vec12.as_ptr().cast::<u8>();
                            let len12 = vec12.len();
                            ::core::mem::forget(vec12);
                            *base.add(12).cast::<usize>() = len12;
                            *base.add(8).cast::<*mut u8>() = ptr12.cast_mut();
                        }
                    }
                    *base.add(8).cast::<usize>() = len13;
                    *base.add(4).cast::<*mut u8>() = result13;
                    let vec14 = (message9.into_bytes()).into_boxed_slice();
                    let ptr14 = vec14.as_ptr().cast::<u8>();
                    let len14 = vec14.len();
                    ::core::mem::forget(vec14);
                    *base.add(16).cast::<usize>() = len14;
                    *base.add(12).cast::<*mut u8>() = ptr14.cast_mut();
                }
            };
        }
    }
    *ptr8.add(4).cast::<usize>() = len15;
    *ptr8.add(0).cast::<*mut u8>() = result15;
    ptr8
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_authorize_edge_node_post_execution<T: Guest>(arg0: *mut u8) {
    let l0 = *arg0.add(0).cast::<*mut u8>();
    let l1 = *arg0.add(4).cast::<usize>();
    let base12 = l0;
    let len12 = l1;
    for i in 0..len12 {
        let base = base12.add(i * 20);
        {
            let l2 = i32::from(*base.add(0).cast::<u8>());
            match l2 {
                0 => {}
                _ => {
                    let l3 = *base.add(4).cast::<*mut u8>();
                    let l4 = *base.add(8).cast::<usize>();
                    let base9 = l3;
                    let len9 = l4;
                    for i in 0..len9 {
                        let base = base9.add(i * 16);
                        {
                            let l5 = *base.add(0).cast::<*mut u8>();
                            let l6 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l5, l6, 1);
                            let l7 = *base.add(8).cast::<*mut u8>();
                            let l8 = *base.add(12).cast::<usize>();
                            _rt::cabi_dealloc(l7, l8, 1);
                        }
                    }
                    _rt::cabi_dealloc(base9, len9 * 16, 4);
                    let l10 = *base.add(12).cast::<*mut u8>();
                    let l11 = *base.add(16).cast::<usize>();
                    _rt::cabi_dealloc(l10, l11, 1);
                }
            }
        }
    }
    _rt::cabi_dealloc(base12, len12 * 20, 4);
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_authorize_edge_post_execution_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
    arg5: *mut u8,
    arg6: usize,
    arg7: *mut u8,
    arg8: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let base11 = arg5;
    let len11 = arg6;
    let mut result11 = _rt::Vec::with_capacity(len11);
    for i in 0..len11 {
        let base = base11.add(i * 16);
        let e11 = {
            let l2 = *base.add(0).cast::<*mut u8>();
            let l3 = *base.add(4).cast::<usize>();
            let len4 = l3;
            let bytes4 = _rt::Vec::from_raw_parts(l2.cast(), len4, len4);
            let l5 = *base.add(8).cast::<*mut u8>();
            let l6 = *base.add(12).cast::<usize>();
            let base10 = l5;
            let len10 = l6;
            let mut result10 = _rt::Vec::with_capacity(len10);
            for i in 0..len10 {
                let base = base10.add(i * 8);
                let e10 = {
                    let l7 = *base.add(0).cast::<*mut u8>();
                    let l8 = *base.add(4).cast::<usize>();
                    let len9 = l8;
                    let bytes9 = _rt::Vec::from_raw_parts(l7.cast(), len9, len9);
                    _rt::string_lift(bytes9)
                };
                result10.push(e10);
            }
            _rt::cabi_dealloc(base10, len10 * 8, 4);
            (_rt::string_lift(bytes4), result10)
        };
        result11.push(e11);
    }
    _rt::cabi_dealloc(base11, len11 * 16, 4);
    let len12 = arg8;
    let bytes12 = _rt::Vec::from_raw_parts(arg7.cast(), len12, len12);
    let result13 = T::authorize_edge_post_execution(
        SharedContext::from_handle(arg0 as u32),
        EdgeDefinition {
            parent_type_name: _rt::string_lift(bytes0),
            field_name: _rt::string_lift(bytes1),
        },
        result11,
        _rt::string_lift(bytes12),
    );
    let ptr14 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    let vec21 = result13;
    let len21 = vec21.len();
    let layout21 = _rt::alloc::Layout::from_size_align_unchecked(vec21.len() * 20, 4);
    let result21 = if layout21.size() != 0 {
        let ptr = _rt::alloc::alloc(layout21).cast::<u8>();
        if ptr.is_null() {
            _rt::alloc::handle_alloc_error(layout21);
        }
        ptr
    } else {
        ::core::ptr::null_mut()
    };
    for (i, e) in vec21.into_iter().enumerate() {
        let base = result21.add(i * 20);
        {
            match e {
                Ok(_) => {
                    *base.add(0).cast::<u8>() = (0i32) as u8;
                }
                Err(e) => {
                    *base.add(0).cast::<u8>() = (1i32) as u8;
                    let Error { extensions: extensions15, message: message15 } = e;
                    let vec19 = extensions15;
                    let len19 = vec19.len();
                    let layout19 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec19.len() * 16,
                        4,
                    );
                    let result19 = if layout19.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout19).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout19);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec19.into_iter().enumerate() {
                        let base = result19.add(i * 16);
                        {
                            let (t16_0, t16_1) = e;
                            let vec17 = (t16_0.into_bytes()).into_boxed_slice();
                            let ptr17 = vec17.as_ptr().cast::<u8>();
                            let len17 = vec17.len();
                            ::core::mem::forget(vec17);
                            *base.add(4).cast::<usize>() = len17;
                            *base.add(0).cast::<*mut u8>() = ptr17.cast_mut();
                            let vec18 = (t16_1.into_bytes()).into_boxed_slice();
                            let ptr18 = vec18.as_ptr().cast::<u8>();
                            let len18 = vec18.len();
                            ::core::mem::forget(vec18);
                            *base.add(12).cast::<usize>() = len18;
                            *base.add(8).cast::<*mut u8>() = ptr18.cast_mut();
                        }
                    }
                    *base.add(8).cast::<usize>() = len19;
                    *base.add(4).cast::<*mut u8>() = result19;
                    let vec20 = (message15.into_bytes()).into_boxed_slice();
                    let ptr20 = vec20.as_ptr().cast::<u8>();
                    let len20 = vec20.len();
                    ::core::mem::forget(vec20);
                    *base.add(16).cast::<usize>() = len20;
                    *base.add(12).cast::<*mut u8>() = ptr20.cast_mut();
                }
            };
        }
    }
    *ptr14.add(4).cast::<usize>() = len21;
    *ptr14.add(0).cast::<*mut u8>() = result21;
    ptr14
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_authorize_edge_post_execution<T: Guest>(arg0: *mut u8) {
    let l0 = *arg0.add(0).cast::<*mut u8>();
    let l1 = *arg0.add(4).cast::<usize>();
    let base12 = l0;
    let len12 = l1;
    for i in 0..len12 {
        let base = base12.add(i * 20);
        {
            let l2 = i32::from(*base.add(0).cast::<u8>());
            match l2 {
                0 => {}
                _ => {
                    let l3 = *base.add(4).cast::<*mut u8>();
                    let l4 = *base.add(8).cast::<usize>();
                    let base9 = l3;
                    let len9 = l4;
                    for i in 0..len9 {
                        let base = base9.add(i * 16);
                        {
                            let l5 = *base.add(0).cast::<*mut u8>();
                            let l6 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l5, l6, 1);
                            let l7 = *base.add(8).cast::<*mut u8>();
                            let l8 = *base.add(12).cast::<usize>();
                            _rt::cabi_dealloc(l7, l8, 1);
                        }
                    }
                    _rt::cabi_dealloc(base9, len9 * 16, 4);
                    let l10 = *base.add(12).cast::<*mut u8>();
                    let l11 = *base.add(16).cast::<usize>();
                    _rt::cabi_dealloc(l10, l11, 1);
                }
            }
        }
    }
    _rt::cabi_dealloc(base12, len12 * 20, 4);
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_on_subgraph_response_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
    arg5: *mut u8,
    arg6: usize,
    arg7: *mut u8,
    arg8: usize,
    arg9: i32,
    arg10: i64,
    arg11: i32,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let len2 = arg6;
    let bytes2 = _rt::Vec::from_raw_parts(arg5.cast(), len2, len2);
    let base8 = arg7;
    let len8 = arg8;
    let mut result8 = _rt::Vec::with_capacity(len8);
    for i in 0..len8 {
        let base = base8.add(i * 32);
        let e8 = {
            let l3 = i32::from(*base.add(0).cast::<u8>());
            let v7 = match l3 {
                0 => SubgraphRequestExecutionKind::InternalServerError,
                1 => SubgraphRequestExecutionKind::HookError,
                2 => SubgraphRequestExecutionKind::RequestError,
                3 => SubgraphRequestExecutionKind::RateLimited,
                n => {
                    debug_assert_eq!(n, 4, "invalid enum discriminant");
                    let e7 = {
                        let l4 = *base.add(8).cast::<i64>();
                        let l5 = *base.add(16).cast::<i64>();
                        let l6 = i32::from(*base.add(24).cast::<u16>());
                        SubgraphResponse {
                            connection_time_ms: l4 as u64,
                            response_time_ms: l5 as u64,
                            status_code: l6 as u16,
                        }
                    };
                    SubgraphRequestExecutionKind::Response(e7)
                }
            };
            v7
        };
        result8.push(e8);
    }
    _rt::cabi_dealloc(base8, len8 * 32, 8);
    let result9 = T::on_subgraph_response(
        SharedContext::from_handle(arg0 as u32),
        ExecutedSubgraphRequest {
            subgraph_name: _rt::string_lift(bytes0),
            method: _rt::string_lift(bytes1),
            url: _rt::string_lift(bytes2),
            executions: result8,
            cache_status: CacheStatus::_lift(arg9 as u8),
            total_duration_ms: arg10 as u64,
            has_errors: _rt::bool_lift(arg11 as u8),
        },
    );
    let ptr10 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    let vec11 = (result9).into_boxed_slice();
    let ptr11 = vec11.as_ptr().cast::<u8>();
    let len11 = vec11.len();
    ::core::mem::forget(vec11);
    *ptr10.add(4).cast::<usize>() = len11;
    *ptr10.add(0).cast::<*mut u8>() = ptr11.cast_mut();
    ptr10
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_on_subgraph_response<T: Guest>(arg0: *mut u8) {
    let l0 = *arg0.add(0).cast::<*mut u8>();
    let l1 = *arg0.add(4).cast::<usize>();
    let base2 = l0;
    let len2 = l1;
    _rt::cabi_dealloc(base2, len2 * 1, 1);
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_on_operation_response_cabi<T: Guest>(
    arg0: i32,
    arg1: i32,
    arg2: *mut u8,
    arg3: usize,
    arg4: *mut u8,
    arg5: usize,
    arg6: i64,
    arg7: i32,
    arg8: i64,
    arg9: i32,
    arg10: i64,
    arg11: i32,
    arg12: *mut u8,
    arg13: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len1 = arg5;
    let bytes1 = _rt::Vec::from_raw_parts(arg4.cast(), len1, len1);
    let v2 = match arg9 {
        0 => GraphqlResponseStatus::Success,
        1 => {
            let e2 = FieldError {
                count: arg10 as u64,
                data_is_null: _rt::bool_lift(arg11 as u8),
            };
            GraphqlResponseStatus::FieldError(e2)
        }
        2 => {
            let e2 = RequestError {
                count: arg10 as u64,
            };
            GraphqlResponseStatus::RequestError(e2)
        }
        n => {
            debug_assert_eq!(n, 3, "invalid enum discriminant");
            GraphqlResponseStatus::RefusedRequest
        }
    };
    let base6 = arg12;
    let len6 = arg13;
    let mut result6 = _rt::Vec::with_capacity(len6);
    for i in 0..len6 {
        let base = base6.add(i * 8);
        let e6 = {
            let l3 = *base.add(0).cast::<*mut u8>();
            let l4 = *base.add(4).cast::<usize>();
            let len5 = l4;
            _rt::Vec::from_raw_parts(l3.cast(), len5, len5)
        };
        result6.push(e6);
    }
    _rt::cabi_dealloc(base6, len6 * 8, 4);
    let result7 = T::on_operation_response(
        SharedContext::from_handle(arg0 as u32),
        ExecutedOperation {
            name: match arg1 {
                0 => None,
                1 => {
                    let e = {
                        let len0 = arg3;
                        let bytes0 = _rt::Vec::from_raw_parts(arg2.cast(), len0, len0);
                        _rt::string_lift(bytes0)
                    };
                    Some(e)
                }
                _ => _rt::invalid_enum_discriminant(),
            },
            document: _rt::string_lift(bytes1),
            prepare_duration_ms: arg6 as u64,
            cached_plan: _rt::bool_lift(arg7 as u8),
            duration_ms: arg8 as u64,
            status: v2,
            on_subgraph_response_outputs: result6,
        },
    );
    let ptr8 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    let vec9 = (result7).into_boxed_slice();
    let ptr9 = vec9.as_ptr().cast::<u8>();
    let len9 = vec9.len();
    ::core::mem::forget(vec9);
    *ptr8.add(4).cast::<usize>() = len9;
    *ptr8.add(0).cast::<*mut u8>() = ptr9.cast_mut();
    ptr8
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_on_operation_response<T: Guest>(arg0: *mut u8) {
    let l0 = *arg0.add(0).cast::<*mut u8>();
    let l1 = *arg0.add(4).cast::<usize>();
    let base2 = l0;
    let len2 = l1;
    _rt::cabi_dealloc(base2, len2 * 1, 1);
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_on_http_response_cabi<T: Guest>(
    arg0: i32,
    arg1: *mut u8,
    arg2: usize,
    arg3: *mut u8,
    arg4: usize,
    arg5: i32,
    arg6: *mut u8,
    arg7: usize,
) {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg2;
    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
    let len1 = arg4;
    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
    let base5 = arg6;
    let len5 = arg7;
    let mut result5 = _rt::Vec::with_capacity(len5);
    for i in 0..len5 {
        let base = base5.add(i * 8);
        let e5 = {
            let l2 = *base.add(0).cast::<*mut u8>();
            let l3 = *base.add(4).cast::<usize>();
            let len4 = l3;
            _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
        };
        result5.push(e5);
    }
    _rt::cabi_dealloc(base5, len5 * 8, 4);
    T::on_http_response(
        SharedContext::from_handle(arg0 as u32),
        ExecutedHttpRequest {
            method: _rt::string_lift(bytes0),
            url: _rt::string_lift(bytes1),
            status_code: arg5 as u16,
            on_operation_response_outputs: result5,
        },
    );
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_init_hooks_cabi<T: Guest>() -> i64 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let result0 = T::init_hooks();
    _rt::as_i64(result0)
}
pub trait Guest {
    /// The hook is called in the federated gateway just before authentication. It can be used
    /// to read and modify the request headers. The context object is provided in a mutable form,
    /// allowing storage for the subsequent hooks to read.
    ///
    /// If returning an error from the hook, the request processing is stopped and the given error
    /// returned to the client.
    fn on_gateway_request(
        context: Context,
        headers: Headers,
    ) -> Result<(), ErrorResponse>;
    /// The hook is called just before requesting a subgraph, after rate limiting is done. It can be used
    /// to read and modify the subgraph request headers. If returning an error, the subgraph is not requested.
    fn on_subgraph_request(
        context: SharedContext,
        subgraph_name: _rt::String,
        method: HttpMethod,
        url: _rt::String,
        headers: Headers,
    ) -> Result<(), Error>;
    /// The hook is called in the request cycle if the schema defines an authorization directive on
    /// an edge, providing the arguments of the edge selected in the directive, the definition of the esge
    /// and the metadata of the directive to the hook.
    ///
    /// The hook is run before fetching any data.
    ///
    /// The result, if an error, will stop the request execution and return an error back to the user.
    /// Result of the edge will be null for an error response.
    fn authorize_edge_pre_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        arguments: _rt::String,
        metadata: _rt::String,
    ) -> Result<(), Error>;
    /// The hook is called in the request cycle if the schema defines an authorization directive to
    /// a node, providing the definition of the node and the metadata of the directive to the hook.
    ///
    /// The hook is run before fetching any data.
    ///
    /// The result, if an error, will stop the request execution and return an error back to the user.
    /// Result of the edge will be null for an error response.
    fn authorize_node_pre_execution(
        context: SharedContext,
        definition: NodeDefinition,
        metadata: _rt::String,
    ) -> Result<(), Error>;
    /// The hook is called in the request cycle if the schema defines an authorization directive on
    /// an edge with the fields argument, providing fields from the parent node. The hook gets the
    /// parent type information, and a list of data with the defined fields of the parent for every
    /// child loaded by the parent query.
    ///
    /// The hook is run after fetching the data.
    ///
    /// The result can be one of the following:
    ///
    /// - A list of one item, which dictates the result for every child loaded from the edge
    /// - A list of many items, each one defining if the child should be shown or not
    ///
    /// Providing any other response will lead to the whole authorization hook failing and data not
    /// returned to the user.
    ///
    /// The list item can either be an empty Ok, which returns the edge data to the client. Or the
    /// item can be an error and the edge access is denied. The error data will be propagated to the
    /// response errors.
    fn authorize_parent_edge_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        parents: _rt::Vec<_rt::String>,
        metadata: _rt::String,
    ) -> _rt::Vec<Result<(), Error>>;
    /// The hook is called in the request cycle if the schema defines an authorization directive on
    /// an edge with the node argument, providing fields from the child node. The hook gets the parent type information,
    /// and a list of data with the defined fields for every child loaded by the parent query.
    ///
    /// The hook is run after fetching the data.
    ///
    /// The result can be one of the following:
    ///
    /// - A list of one item, which dictates the result for every child loaded from the edge
    /// - A list of many items, each one defining if the child should be shown or not
    ///
    /// Providing any other response will lead to the whole authorization hook failing and data not
    /// returned to the user.
    ///
    /// The list item can either be an empty Ok, which returns the edge data to the client. Or the
    /// item can be an error and the edge access is denied. The error data will be propagated to the
    /// response errors.
    fn authorize_edge_node_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        nodes: _rt::Vec<_rt::String>,
        metadata: _rt::String,
    ) -> _rt::Vec<Result<(), Error>>;
    /// The hook is called in the request cycle if the schema defines an authorization directive on
    /// an edge with the node and fields arguments, providing fields from the child node. The hook gets
    /// the parent type information, and a list of data with tuples of the parent data and a list of child data.
    ///
    /// The first part of the tuple is defined by the directive's fields argument and the second part by
    /// the node argument.
    ///
    /// The hook is run after fetching the data.
    ///
    /// The result can be one of the following:
    ///
    /// - A list of one item, which dictates the result for every child loaded from the edge
    /// - A list of many items, each one defining if the child should be shown or not
    ///
    /// Providing any other response will lead to the whole authorization hook failing and data not
    /// returned to the user.
    ///
    /// The list item can either be an empty Ok, which returns the edge data to the client. Or the
    /// item can be an error and the edge access is denied. The error data will be propagated to the
    /// response errors.
    fn authorize_edge_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        edges: _rt::Vec<(_rt::String, _rt::Vec<_rt::String>)>,
        metadata: _rt::String,
    ) -> _rt::Vec<Result<(), Error>>;
    /// The hook is called after a subgraph entity has been either requested or fetched from cache.
    /// The output is a list of bytes, which will be available in the on-operation-response hook.
    fn on_subgraph_response(
        context: SharedContext,
        request: ExecutedSubgraphRequest,
    ) -> _rt::Vec<u8>;
    /// The hook is called after a request is handled in the gateway. The output is a list of bytes,
    /// which will be available in the on-http-response hook.
    fn on_operation_response(
        context: SharedContext,
        request: ExecutedOperation,
    ) -> _rt::Vec<u8>;
    /// The hook is called right before a response is sent to the user.
    fn on_http_response(context: SharedContext, request: ExecutedHttpRequest);
    /// The hooks initialization function. Must be called before any other hook function.
    fn init_hooks() -> i64;
}
#[doc(hidden)]
macro_rules! __export_world_hooks_cabi {
    ($ty:ident with_types_in $($path_to_types:tt)*) => {
        const _ : () = { #[export_name = "on-gateway-request"] unsafe extern "C" fn
        export_on_gateway_request(arg0 : i32, arg1 : i32,) -> * mut u8 {
        $($path_to_types)*:: _export_on_gateway_request_cabi::<$ty > (arg0, arg1) }
        #[export_name = "cabi_post_on-gateway-request"] unsafe extern "C" fn
        _post_return_on_gateway_request(arg0 : * mut u8,) { $($path_to_types)*::
        __post_return_on_gateway_request::<$ty > (arg0) } #[export_name =
        "on-subgraph-request"] unsafe extern "C" fn export_on_subgraph_request(arg0 :
        i32, arg1 : * mut u8, arg2 : usize, arg3 : i32, arg4 : * mut u8, arg5 : usize,
        arg6 : i32,) -> * mut u8 { $($path_to_types)*::
        _export_on_subgraph_request_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4, arg5,
        arg6) } #[export_name = "cabi_post_on-subgraph-request"] unsafe extern "C" fn
        _post_return_on_subgraph_request(arg0 : * mut u8,) { $($path_to_types)*::
        __post_return_on_subgraph_request::<$ty > (arg0) } #[export_name =
        "authorize-edge-pre-execution"] unsafe extern "C" fn
        export_authorize_edge_pre_execution(arg0 : i32, arg1 : * mut u8, arg2 : usize,
        arg3 : * mut u8, arg4 : usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut u8,
        arg8 : usize,) -> * mut u8 { $($path_to_types)*::
        _export_authorize_edge_pre_execution_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4,
        arg5, arg6, arg7, arg8) } #[export_name =
        "cabi_post_authorize-edge-pre-execution"] unsafe extern "C" fn
        _post_return_authorize_edge_pre_execution(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_authorize_edge_pre_execution::<$ty > (arg0) }
        #[export_name = "authorize-node-pre-execution"] unsafe extern "C" fn
        export_authorize_node_pre_execution(arg0 : i32, arg1 : * mut u8, arg2 : usize,
        arg3 : * mut u8, arg4 : usize,) -> * mut u8 { $($path_to_types)*::
        _export_authorize_node_pre_execution_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4)
        } #[export_name = "cabi_post_authorize-node-pre-execution"] unsafe extern "C" fn
        _post_return_authorize_node_pre_execution(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_authorize_node_pre_execution::<$ty > (arg0) }
        #[export_name = "authorize-parent-edge-post-execution"] unsafe extern "C" fn
        export_authorize_parent_edge_post_execution(arg0 : i32, arg1 : * mut u8, arg2 :
        usize, arg3 : * mut u8, arg4 : usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut
        u8, arg8 : usize,) -> * mut u8 { $($path_to_types)*::
        _export_authorize_parent_edge_post_execution_cabi::<$ty > (arg0, arg1, arg2,
        arg3, arg4, arg5, arg6, arg7, arg8) } #[export_name =
        "cabi_post_authorize-parent-edge-post-execution"] unsafe extern "C" fn
        _post_return_authorize_parent_edge_post_execution(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_authorize_parent_edge_post_execution::<$ty >
        (arg0) } #[export_name = "authorize-edge-node-post-execution"] unsafe extern "C"
        fn export_authorize_edge_node_post_execution(arg0 : i32, arg1 : * mut u8, arg2 :
        usize, arg3 : * mut u8, arg4 : usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut
        u8, arg8 : usize,) -> * mut u8 { $($path_to_types)*::
        _export_authorize_edge_node_post_execution_cabi::<$ty > (arg0, arg1, arg2, arg3,
        arg4, arg5, arg6, arg7, arg8) } #[export_name =
        "cabi_post_authorize-edge-node-post-execution"] unsafe extern "C" fn
        _post_return_authorize_edge_node_post_execution(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_authorize_edge_node_post_execution::<$ty >
        (arg0) } #[export_name = "authorize-edge-post-execution"] unsafe extern "C" fn
        export_authorize_edge_post_execution(arg0 : i32, arg1 : * mut u8, arg2 : usize,
        arg3 : * mut u8, arg4 : usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut u8,
        arg8 : usize,) -> * mut u8 { $($path_to_types)*::
        _export_authorize_edge_post_execution_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4,
        arg5, arg6, arg7, arg8) } #[export_name =
        "cabi_post_authorize-edge-post-execution"] unsafe extern "C" fn
        _post_return_authorize_edge_post_execution(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_authorize_edge_post_execution::<$ty > (arg0) }
        #[export_name = "on-subgraph-response"] unsafe extern "C" fn
        export_on_subgraph_response(arg0 : i32, arg1 : * mut u8, arg2 : usize, arg3 : *
        mut u8, arg4 : usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut u8, arg8 :
        usize, arg9 : i32, arg10 : i64, arg11 : i32,) -> * mut u8 { $($path_to_types)*::
        _export_on_subgraph_response_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4, arg5,
        arg6, arg7, arg8, arg9, arg10, arg11) } #[export_name =
        "cabi_post_on-subgraph-response"] unsafe extern "C" fn
        _post_return_on_subgraph_response(arg0 : * mut u8,) { $($path_to_types)*::
        __post_return_on_subgraph_response::<$ty > (arg0) } #[export_name =
        "on-operation-response"] unsafe extern "C" fn export_on_operation_response(arg0 :
        i32, arg1 : i32, arg2 : * mut u8, arg3 : usize, arg4 : * mut u8, arg5 : usize,
        arg6 : i64, arg7 : i32, arg8 : i64, arg9 : i32, arg10 : i64, arg11 : i32, arg12 :
        * mut u8, arg13 : usize,) -> * mut u8 { $($path_to_types)*::
        _export_on_operation_response_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4, arg5,
        arg6, arg7, arg8, arg9, arg10, arg11, arg12, arg13) } #[export_name =
        "cabi_post_on-operation-response"] unsafe extern "C" fn
        _post_return_on_operation_response(arg0 : * mut u8,) { $($path_to_types)*::
        __post_return_on_operation_response::<$ty > (arg0) } #[export_name =
        "on-http-response"] unsafe extern "C" fn export_on_http_response(arg0 : i32, arg1
        : * mut u8, arg2 : usize, arg3 : * mut u8, arg4 : usize, arg5 : i32, arg6 : * mut
        u8, arg7 : usize,) { $($path_to_types)*:: _export_on_http_response_cabi::<$ty >
        (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) } #[export_name = "init-hooks"]
        unsafe extern "C" fn export_init_hooks() -> i64 { $($path_to_types)*::
        _export_init_hooks_cabi::<$ty > () } };
    };
}
#[doc(hidden)]
pub(crate) use __export_world_hooks_cabi;
#[repr(align(4))]
struct _RetArea([::core::mem::MaybeUninit<u8>; 20]);
static mut _RET_AREA: _RetArea = _RetArea([::core::mem::MaybeUninit::uninit(); 20]);
mod _rt {
    pub use alloc_crate::vec::Vec;
    use core::fmt;
    use core::marker;
    use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
    /// A type which represents a component model resource, either imported or
    /// exported into this component.
    ///
    /// This is a low-level wrapper which handles the lifetime of the resource
    /// (namely this has a destructor). The `T` provided defines the component model
    /// intrinsics that this wrapper uses.
    ///
    /// One of the chief purposes of this type is to provide `Deref` implementations
    /// to access the underlying data when it is owned.
    ///
    /// This type is primarily used in generated code for exported and imported
    /// resources.
    #[repr(transparent)]
    pub struct Resource<T: WasmResource> {
        handle: AtomicU32,
        _marker: marker::PhantomData<T>,
    }
    /// A trait which all wasm resources implement, namely providing the ability to
    /// drop a resource.
    ///
    /// This generally is implemented by generated code, not user-facing code.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait WasmResource {
        /// Invokes the `[resource-drop]...` intrinsic.
        unsafe fn drop(handle: u32);
    }
    impl<T: WasmResource> Resource<T> {
        #[doc(hidden)]
        pub unsafe fn from_handle(handle: u32) -> Self {
            debug_assert!(handle != u32::MAX);
            Self {
                handle: AtomicU32::new(handle),
                _marker: marker::PhantomData,
            }
        }
        /// Takes ownership of the handle owned by `resource`.
        ///
        /// Note that this ideally would be `into_handle` taking `Resource<T>` by
        /// ownership. The code generator does not enable that in all situations,
        /// unfortunately, so this is provided instead.
        ///
        /// Also note that `take_handle` is in theory only ever called on values
        /// owned by a generated function. For example a generated function might
        /// take `Resource<T>` as an argument but then call `take_handle` on a
        /// reference to that argument. In that sense the dynamic nature of
        /// `take_handle` should only be exposed internally to generated code, not
        /// to user code.
        #[doc(hidden)]
        pub fn take_handle(resource: &Resource<T>) -> u32 {
            resource.handle.swap(u32::MAX, Relaxed)
        }
        #[doc(hidden)]
        pub fn handle(resource: &Resource<T>) -> u32 {
            resource.handle.load(Relaxed)
        }
    }
    impl<T: WasmResource> fmt::Debug for Resource<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Resource").field("handle", &self.handle).finish()
        }
    }
    impl<T: WasmResource> Drop for Resource<T> {
        fn drop(&mut self) {
            unsafe {
                match self.handle.load(Relaxed) {
                    u32::MAX => {}
                    other => T::drop(other),
                }
            }
        }
    }
    pub use alloc_crate::string::String;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub use alloc_crate::alloc;
    pub fn as_i64<T: AsI64>(t: T) -> i64 {
        t.as_i64()
    }
    pub trait AsI64 {
        fn as_i64(self) -> i64;
    }
    impl<'a, T: Copy + AsI64> AsI64 for &'a T {
        fn as_i64(self) -> i64 {
            (*self).as_i64()
        }
    }
    impl AsI64 for i64 {
        #[inline]
        fn as_i64(self) -> i64 {
            self as i64
        }
    }
    impl AsI64 for u64 {
        #[inline]
        fn as_i64(self) -> i64 {
            self as i64
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub fn as_i32<T: AsI32>(t: T) -> i32 {
        t.as_i32()
    }
    pub trait AsI32 {
        fn as_i32(self) -> i32;
    }
    impl<'a, T: Copy + AsI32> AsI32 for &'a T {
        fn as_i32(self) -> i32 {
            (*self).as_i32()
        }
    }
    impl AsI32 for i32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for char {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for usize {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    pub unsafe fn bool_lift(val: u8) -> bool {
        if cfg!(debug_assertions) {
            match val {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
            }
        } else {
            val != 0
        }
    }
    extern crate alloc as alloc_crate;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_hooks_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*:: __export_world_hooks_cabi!($ty with_types_in
        $($path_to_types_root)*);
    };
}
#[doc(inline)]
pub(crate) use __export_hooks_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.31.0:component:grafbase:hooks:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 2843] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x9f\x15\x01A\x02\x01\
Ar\x01m\x02\x14invalid-header-value\x13invalid-header-name\x03\0\x0cheader-error\
\x03\0\0\x01p}\x01q\x02\x0cchannel-full\x01\x02\0\x0echannel-closed\0\0\x03\0\x09\
log-error\x03\0\x03\x03\0\x07context\x03\x01\x03\0\x0eshared-context\x03\x01\x03\
\0\x07headers\x03\x01\x01r\x02\x10parent-type-names\x0afield-names\x03\0\x0fedge\
-definition\x03\0\x08\x01r\x01\x09type-names\x03\0\x0fnode-definition\x03\0\x0a\x01\
p\x02\x01r\x04\x06methods\x03urls\x0bstatus-code{\x1don-operation-response-outpu\
ts\x0c\x03\0\x15executed-http-request\x03\0\x0d\x01r\x02\x05countw\x0cdata-is-nu\
ll\x7f\x03\0\x0bfield-error\x03\0\x0f\x01r\x01\x05countw\x03\0\x0drequest-error\x03\
\0\x11\x01q\x04\x07success\0\0\x0bfield-error\x01\x10\0\x0drequest-error\x01\x12\
\0\x0frefused-request\0\0\x03\0\x17graphql-response-status\x03\0\x13\x01ks\x01r\x07\
\x04name\x15\x08documents\x13prepare-duration-msw\x0bcached-plan\x7f\x0bduration\
-msw\x06status\x14\x1con-subgraph-response-outputs\x0c\x03\0\x12executed-operati\
on\x03\0\x16\x01r\x03\x12connection-time-msw\x10response-time-msw\x0bstatus-code\
{\x03\0\x11subgraph-response\x03\0\x18\x01m\x03\x03hit\x0bpartial-hit\x04miss\x03\
\0\x0ccache-status\x03\0\x1a\x01q\x05\x15internal-server-error\0\0\x0ahook-error\
\0\0\x0drequest-error\0\0\x0crate-limited\0\0\x08response\x01\x19\0\x03\0\x1fsub\
graph-request-execution-kind\x03\0\x1c\x01p\x1d\x01r\x07\x0dsubgraph-names\x06me\
thods\x03urls\x0aexecutions\x1e\x0ccache-status\x1b\x11total-duration-msw\x0ahas\
-errors\x7f\x03\0\x19executed-subgraph-request\x03\0\x1f\x01o\x02ss\x01p!\x01r\x02\
\x0aextensions\"\x07messages\x03\0\x05error\x03\0#\x01p$\x01r\x02\x0bstatus-code\
{\x06errors%\x03\0\x0eerror-response\x03\0&\x03\0\x0bhttp-client\x03\x01\x03\0\x0a\
access-log\x03\x01\x01m\x09\x03get\x04post\x03put\x06delete\x05patch\x04head\x07\
options\x07connect\x05trace\x03\0\x0bhttp-method\x03\0*\x01kw\x01r\x05\x06method\
+\x03urls\x07headers\"\x04body\x02\x0atimeout-ms,\x03\0\x0chttp-request\x03\0-\x01\
m\x05\x06http09\x06http10\x06http11\x06http20\x06http30\x03\0\x0chttp-version\x03\
\0/\x01r\x04\x06status{\x07version0\x07headers\"\x04body\x02\x03\0\x0dhttp-respo\
nse\x03\01\x01q\x03\x07timeout\0\0\x07request\x01s\0\x07connect\x01s\0\x03\0\x0a\
http-error\x03\03\x01h\x05\x01@\x02\x04self5\x04names\0\x15\x03\0\x13[method]con\
text.get\x016\x01@\x03\x04self5\x04names\x05values\x01\0\x03\0\x13[method]contex\
t.set\x017\x03\0\x16[method]context.delete\x016\x01h\x06\x01@\x02\x04self8\x04na\
mes\0\x15\x03\0\x1a[method]shared-context.get\x019\x01@\x01\x04self8\0s\x03\0\x1f\
[method]shared-context.trace-id\x01:\x01h\x07\x01@\x02\x04self;\x04names\0\x15\x03\
\0\x13[method]headers.get\x01<\x01j\0\x01\x01\x01@\x03\x04self;\x04names\x05valu\
es\0=\x03\0\x13[method]headers.set\x01>\x03\0\x16[method]headers.delete\x01<\x01\
@\x01\x04self;\0\"\x03\0\x17[method]headers.entries\x01?\x01j\x012\x014\x01@\x01\
\x07request.\0\xc0\0\x03\0\x1b[static]http-client.execute\x01A\x01p.\x01p\xc0\0\x01\
@\x01\x08requests\xc2\0\0\xc3\0\x03\0\x20[static]http-client.execute-many\x01D\x01\
j\0\x01\x04\x01@\x01\x04data\x02\0\xc5\0\x03\0\x17[static]access-log.send\x01F\x01\
i\x05\x01i\x07\x01j\0\x01'\x01@\x02\x07context\xc7\0\x07headers\xc8\0\0\xc9\0\x04\
\0\x12on-gateway-request\x01J\x01i\x06\x01j\0\x01$\x01@\x05\x07context\xcb\0\x0d\
subgraph-names\x06method+\x03urls\x07headers\xc8\0\0\xcc\0\x04\0\x13on-subgraph-\
request\x01M\x01@\x04\x07context\xcb\0\x0adefinition\x09\x09argumentss\x08metada\
tas\0\xcc\0\x04\0\x1cauthorize-edge-pre-execution\x01N\x01@\x03\x07context\xcb\0\
\x0adefinition\x0b\x08metadatas\0\xcc\0\x04\0\x1cauthorize-node-pre-execution\x01\
O\x01ps\x01p\xcc\0\x01@\x04\x07context\xcb\0\x0adefinition\x09\x07parents\xd0\0\x08\
metadatas\0\xd1\0\x04\0$authorize-parent-edge-post-execution\x01R\x01@\x04\x07co\
ntext\xcb\0\x0adefinition\x09\x05nodes\xd0\0\x08metadatas\0\xd1\0\x04\0\"authori\
ze-edge-node-post-execution\x01S\x01o\x02s\xd0\0\x01p\xd4\0\x01@\x04\x07context\xcb\
\0\x0adefinition\x09\x05edges\xd5\0\x08metadatas\0\xd1\0\x04\0\x1dauthorize-edge\
-post-execution\x01V\x01@\x02\x07context\xcb\0\x07request\x20\0\x02\x04\0\x14on-\
subgraph-response\x01W\x01@\x02\x07context\xcb\0\x07request\x17\0\x02\x04\0\x15o\
n-operation-response\x01X\x01@\x02\x07context\xcb\0\x07request\x0e\x01\0\x04\0\x10\
on-http-response\x01Y\x01@\0\0x\x04\0\x0ainit-hooks\x01Z\x04\x01\x18component:gr\
afbase/hooks\x04\0\x0b\x0b\x01\0\x05hooks\x03\0\0\0G\x09producers\x01\x0cprocess\
ed-by\x02\x0dwit-component\x070.216.0\x10wit-bindgen-rust\x060.31.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
