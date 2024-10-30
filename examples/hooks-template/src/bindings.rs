#[allow(dead_code)]
pub mod component {
    #[allow(dead_code)]
    pub mod grafbase {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// Error thrown when accessing the headers. Headers names or values
            /// must not contain any special characters.
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
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
                        HeaderError::InvalidHeaderValue => {
                            "the given header value is not valid"
                        }
                        HeaderError::InvalidHeaderName => {
                            "the given header name is not valid"
                        }
                    }
                }
            }
            impl ::core::fmt::Debug for HeaderError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("HeaderError")
                        .field("code", &(*self as i32))
                        .field("name", &self.name())
                        .field("message", &self.message())
                        .finish()
                }
            }
            impl ::core::fmt::Display for HeaderError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        LogError::ChannelFull(e) => {
                            f.debug_tuple("LogError::ChannelFull").field(e).finish()
                        }
                        LogError::ChannelClosed => {
                            f.debug_tuple("LogError::ChannelClosed").finish()
                        }
                    }
                }
            }
            impl ::core::fmt::Display for LogError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
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
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("NodeDefinition")
                        .field("type-name", &self.type_name)
                        .finish()
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("ExecutedHttpRequest")
                        .field("method", &self.method)
                        .field("url", &self.url)
                        .field("status-code", &self.status_code)
                        .field(
                            "on-operation-response-outputs",
                            &self.on_operation_response_outputs,
                        )
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct FieldError {
                /// The number of errors.
                pub count: u64,
                /// The returned data is null.
                pub data_is_null: bool,
            }
            impl ::core::fmt::Debug for FieldError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("FieldError")
                        .field("count", &self.count)
                        .field("data-is-null", &self.data_is_null)
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct RequestError {
                /// The number of errors.
                pub count: u64,
            }
            impl ::core::fmt::Debug for RequestError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("RequestError").field("count", &self.count).finish()
                }
            }
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        GraphqlResponseStatus::Success => {
                            f.debug_tuple("GraphqlResponseStatus::Success").finish()
                        }
                        GraphqlResponseStatus::FieldError(e) => {
                            f.debug_tuple("GraphqlResponseStatus::FieldError")
                                .field(e)
                                .finish()
                        }
                        GraphqlResponseStatus::RequestError(e) => {
                            f.debug_tuple("GraphqlResponseStatus::RequestError")
                                .field(e)
                                .finish()
                        }
                        GraphqlResponseStatus::RefusedRequest => {
                            f.debug_tuple("GraphqlResponseStatus::RefusedRequest")
                                .finish()
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("ExecutedOperation")
                        .field("name", &self.name)
                        .field("document", &self.document)
                        .field("prepare-duration-ms", &self.prepare_duration_ms)
                        .field("cached-plan", &self.cached_plan)
                        .field("duration-ms", &self.duration_ms)
                        .field("status", &self.status)
                        .field(
                            "on-subgraph-response-outputs",
                            &self.on_subgraph_response_outputs,
                        )
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("SubgraphResponse")
                        .field("connection-time-ms", &self.connection_time_ms)
                        .field("response-time-ms", &self.response_time_ms)
                        .field("status-code", &self.status_code)
                        .finish()
                }
            }
            /// Cache status of a subgraph call.
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub enum CacheStatus {
                /// All data fetched from cache.
                Hit,
                /// Some data fetched from cache.
                PartialHit,
                /// Cache miss
                Miss,
            }
            impl ::core::fmt::Debug for CacheStatus {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        CacheStatus::Hit => f.debug_tuple("CacheStatus::Hit").finish(),
                        CacheStatus::PartialHit => {
                            f.debug_tuple("CacheStatus::PartialHit").finish()
                        }
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        SubgraphRequestExecutionKind::InternalServerError => {
                            f.debug_tuple(
                                    "SubgraphRequestExecutionKind::InternalServerError",
                                )
                                .finish()
                        }
                        SubgraphRequestExecutionKind::HookError => {
                            f.debug_tuple("SubgraphRequestExecutionKind::HookError")
                                .finish()
                        }
                        SubgraphRequestExecutionKind::RequestError => {
                            f.debug_tuple("SubgraphRequestExecutionKind::RequestError")
                                .finish()
                        }
                        SubgraphRequestExecutionKind::RateLimited => {
                            f.debug_tuple("SubgraphRequestExecutionKind::RateLimited")
                                .finish()
                        }
                        SubgraphRequestExecutionKind::Response(e) => {
                            f.debug_tuple("SubgraphRequestExecutionKind::Response")
                                .field(e)
                                .finish()
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
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
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Error")
                        .field("extensions", &self.extensions)
                        .field("message", &self.message)
                        .finish()
                }
            }
            impl ::core::fmt::Display for Error {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for Error {}
            impl Context {
                #[allow(unused_unsafe, clippy::all)]
                /// Fetches a context value with the given name, if existing.
                pub fn get(&self, name: &str) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = name;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                                    let bytes5 = _rt::Vec::from_raw_parts(
                                        l3.cast(),
                                        len5,
                                        len5,
                                    );
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
                        #[link(wasm_import_module = "component:grafbase/types")]
                        extern "C" {
                            #[link_name = "[method]context.set"]
                            fn wit_import(
                                _: i32,
                                _: *mut u8,
                                _: usize,
                                _: *mut u8,
                                _: usize,
                            );
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(
                            _: i32,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                        ) {
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
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = name;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                                    let bytes5 = _rt::Vec::from_raw_parts(
                                        l3.cast(),
                                        len5,
                                        len5,
                                    );
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
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = name;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                                    let bytes5 = _rt::Vec::from_raw_parts(
                                        l3.cast(),
                                        len5,
                                        len5,
                                    );
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
                /// Sends the data to the access log.
                pub fn log_access(&self, data: &[u8]) -> Result<(), LogError> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 16],
                        );
                        let vec0 = data;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
                        extern "C" {
                            #[link_name = "[method]shared-context.log-access"]
                            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
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
            impl SharedContext {
                #[allow(unused_unsafe, clippy::all)]
                /// Gets the current trace-id.
                pub fn trace_id(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = name;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                                    let bytes5 = _rt::Vec::from_raw_parts(
                                        l3.cast(),
                                        len5,
                                        len5,
                                    );
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
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 2],
                        );
                        let vec0 = name;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let vec1 = value;
                        let ptr1 = vec1.as_ptr().cast::<u8>();
                        let len1 = vec1.len();
                        let ptr2 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = name;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "component:grafbase/types")]
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
                                    let bytes5 = _rt::Vec::from_raw_parts(
                                        l3.cast(),
                                        len5,
                                        len5,
                                    );
                                    _rt::string_lift(bytes5)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod component {
        #[allow(dead_code)]
        pub mod grafbase {
            #[allow(dead_code, clippy::all)]
            pub mod gateway_request {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type Headers = super::super::super::super::component::grafbase::types::Headers;
                pub type Error = super::super::super::super::component::grafbase::types::Error;
                pub type Context = super::super::super::super::component::grafbase::types::Context;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_on_gateway_request_cabi<T: Guest>(
                    arg0: i32,
                    arg1: i32,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::on_gateway_request(
                        super::super::super::super::component::grafbase::types::Context::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::Headers::from_handle(
                            arg1 as u32,
                        ),
                    );
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result0 {
                        Ok(_) => {
                            *ptr1.add(0).cast::<u8>() = (0i32) as u8;
                        }
                        Err(e) => {
                            *ptr1.add(0).cast::<u8>() = (1i32) as u8;
                            let super::super::super::super::component::grafbase::types::Error {
                                extensions: extensions2,
                                message: message2,
                            } = e;
                            let vec6 = extensions2;
                            let len6 = vec6.len();
                            let layout6 = _rt::alloc::Layout::from_size_align_unchecked(
                                vec6.len() * 16,
                                4,
                            );
                            let result6 = if layout6.size() != 0 {
                                let ptr = _rt::alloc::alloc(layout6).cast::<u8>();
                                if ptr.is_null() {
                                    _rt::alloc::handle_alloc_error(layout6);
                                }
                                ptr
                            } else {
                                { ::core::ptr::null_mut() }
                            };
                            for (i, e) in vec6.into_iter().enumerate() {
                                let base = result6.add(i * 16);
                                {
                                    let (t3_0, t3_1) = e;
                                    let vec4 = (t3_0.into_bytes()).into_boxed_slice();
                                    let ptr4 = vec4.as_ptr().cast::<u8>();
                                    let len4 = vec4.len();
                                    ::core::mem::forget(vec4);
                                    *base.add(4).cast::<usize>() = len4;
                                    *base.add(0).cast::<*mut u8>() = ptr4.cast_mut();
                                    let vec5 = (t3_1.into_bytes()).into_boxed_slice();
                                    let ptr5 = vec5.as_ptr().cast::<u8>();
                                    let len5 = vec5.len();
                                    ::core::mem::forget(vec5);
                                    *base.add(12).cast::<usize>() = len5;
                                    *base.add(8).cast::<*mut u8>() = ptr5.cast_mut();
                                }
                            }
                            *ptr1.add(8).cast::<usize>() = len6;
                            *ptr1.add(4).cast::<*mut u8>() = result6;
                            let vec7 = (message2.into_bytes()).into_boxed_slice();
                            let ptr7 = vec7.as_ptr().cast::<u8>();
                            let len7 = vec7.len();
                            ::core::mem::forget(vec7);
                            *ptr1.add(16).cast::<usize>() = len7;
                            *ptr1.add(12).cast::<*mut u8>() = ptr7.cast_mut();
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
                    ) -> Result<(), Error>;
                }
                #[doc(hidden)]
                macro_rules! __export_component_grafbase_gateway_request_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "component:grafbase/gateway-request#on-gateway-request"] unsafe
                        extern "C" fn export_on_gateway_request(arg0 : i32, arg1 : i32,)
                        -> * mut u8 { $($path_to_types)*::
                        _export_on_gateway_request_cabi::<$ty > (arg0, arg1) }
                        #[export_name =
                        "cabi_post_component:grafbase/gateway-request#on-gateway-request"]
                        unsafe extern "C" fn _post_return_on_gateway_request(arg0 : * mut
                        u8,) { $($path_to_types)*::
                        __post_return_on_gateway_request::<$ty > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_grafbase_gateway_request_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 20]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 20],
                );
            }
            #[allow(dead_code, clippy::all)]
            pub mod subgraph_request {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type SharedContext = super::super::super::super::component::grafbase::types::SharedContext;
                pub type Headers = super::super::super::super::component::grafbase::types::Headers;
                pub type Error = super::super::super::super::component::grafbase::types::Error;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_on_subgraph_request_cabi<T: Guest>(
                    arg0: i32,
                    arg1: *mut u8,
                    arg2: usize,
                    arg3: *mut u8,
                    arg4: usize,
                    arg5: *mut u8,
                    arg6: usize,
                    arg7: i32,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg2;
                    let bytes0 = _rt::Vec::from_raw_parts(arg1.cast(), len0, len0);
                    let len1 = arg4;
                    let bytes1 = _rt::Vec::from_raw_parts(arg3.cast(), len1, len1);
                    let len2 = arg6;
                    let bytes2 = _rt::Vec::from_raw_parts(arg5.cast(), len2, len2);
                    let result3 = T::on_subgraph_request(
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        _rt::string_lift(bytes0),
                        _rt::string_lift(bytes1),
                        _rt::string_lift(bytes2),
                        super::super::super::super::component::grafbase::types::Headers::from_handle(
                            arg7 as u32,
                        ),
                    );
                    let ptr4 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result3 {
                        Ok(_) => {
                            *ptr4.add(0).cast::<u8>() = (0i32) as u8;
                        }
                        Err(e) => {
                            *ptr4.add(0).cast::<u8>() = (1i32) as u8;
                            let super::super::super::super::component::grafbase::types::Error {
                                extensions: extensions5,
                                message: message5,
                            } = e;
                            let vec9 = extensions5;
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
                                { ::core::ptr::null_mut() }
                            };
                            for (i, e) in vec9.into_iter().enumerate() {
                                let base = result9.add(i * 16);
                                {
                                    let (t6_0, t6_1) = e;
                                    let vec7 = (t6_0.into_bytes()).into_boxed_slice();
                                    let ptr7 = vec7.as_ptr().cast::<u8>();
                                    let len7 = vec7.len();
                                    ::core::mem::forget(vec7);
                                    *base.add(4).cast::<usize>() = len7;
                                    *base.add(0).cast::<*mut u8>() = ptr7.cast_mut();
                                    let vec8 = (t6_1.into_bytes()).into_boxed_slice();
                                    let ptr8 = vec8.as_ptr().cast::<u8>();
                                    let len8 = vec8.len();
                                    ::core::mem::forget(vec8);
                                    *base.add(12).cast::<usize>() = len8;
                                    *base.add(8).cast::<*mut u8>() = ptr8.cast_mut();
                                }
                            }
                            *ptr4.add(8).cast::<usize>() = len9;
                            *ptr4.add(4).cast::<*mut u8>() = result9;
                            let vec10 = (message5.into_bytes()).into_boxed_slice();
                            let ptr10 = vec10.as_ptr().cast::<u8>();
                            let len10 = vec10.len();
                            ::core::mem::forget(vec10);
                            *ptr4.add(16).cast::<usize>() = len10;
                            *ptr4.add(12).cast::<*mut u8>() = ptr10.cast_mut();
                        }
                    };
                    ptr4
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_on_subgraph_request<T: Guest>(
                    arg0: *mut u8,
                ) {
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
                pub trait Guest {
                    /// The hook is called just before requesting a subgraph, after rate limiting is done. It can be used
                    /// to read and modify the subgraph request headers. If returning an error, the subgraph is not requested.
                    fn on_subgraph_request(
                        context: SharedContext,
                        subgraph_name: _rt::String,
                        method: _rt::String,
                        url: _rt::String,
                        headers: Headers,
                    ) -> Result<(), Error>;
                }
                #[doc(hidden)]
                macro_rules! __export_component_grafbase_subgraph_request_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "component:grafbase/subgraph-request#on-subgraph-request"] unsafe
                        extern "C" fn export_on_subgraph_request(arg0 : i32, arg1 : * mut
                        u8, arg2 : usize, arg3 : * mut u8, arg4 : usize, arg5 : * mut u8,
                        arg6 : usize, arg7 : i32,) -> * mut u8 { $($path_to_types)*::
                        _export_on_subgraph_request_cabi::<$ty > (arg0, arg1, arg2, arg3,
                        arg4, arg5, arg6, arg7) } #[export_name =
                        "cabi_post_component:grafbase/subgraph-request#on-subgraph-request"]
                        unsafe extern "C" fn _post_return_on_subgraph_request(arg0 : *
                        mut u8,) { $($path_to_types)*::
                        __post_return_on_subgraph_request::<$ty > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_grafbase_subgraph_request_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 20]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 20],
                );
            }
            #[allow(dead_code, clippy::all)]
            pub mod authorization {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type Error = super::super::super::super::component::grafbase::types::Error;
                pub type SharedContext = super::super::super::super::component::grafbase::types::SharedContext;
                pub type EdgeDefinition = super::super::super::super::component::grafbase::types::EdgeDefinition;
                pub type NodeDefinition = super::super::super::super::component::grafbase::types::NodeDefinition;
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::EdgeDefinition {
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
                            let super::super::super::super::component::grafbase::types::Error {
                                extensions: extensions6,
                                message: message6,
                            } = e;
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
                                { ::core::ptr::null_mut() }
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
                pub unsafe fn __post_return_authorize_edge_pre_execution<T: Guest>(
                    arg0: *mut u8,
                ) {
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::NodeDefinition {
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
                            let super::super::super::super::component::grafbase::types::Error {
                                extensions: extensions4,
                                message: message4,
                            } = e;
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
                                { ::core::ptr::null_mut() }
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
                pub unsafe fn __post_return_authorize_node_pre_execution<T: Guest>(
                    arg0: *mut u8,
                ) {
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
                pub unsafe fn _export_authorize_parent_edge_post_execution_cabi<
                    T: Guest,
                >(
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::EdgeDefinition {
                            parent_type_name: _rt::string_lift(bytes0),
                            field_name: _rt::string_lift(bytes1),
                        },
                        result5,
                        _rt::string_lift(bytes6),
                    );
                    let ptr8 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec15 = result7;
                    let len15 = vec15.len();
                    let layout15 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec15.len() * 20,
                        4,
                    );
                    let result15 = if layout15.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout15).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout15);
                        }
                        ptr
                    } else {
                        { ::core::ptr::null_mut() }
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
                                    let super::super::super::super::component::grafbase::types::Error {
                                        extensions: extensions9,
                                        message: message9,
                                    } = e;
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
                                        { ::core::ptr::null_mut() }
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
                pub unsafe fn __post_return_authorize_parent_edge_post_execution<
                    T: Guest,
                >(arg0: *mut u8) {
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::EdgeDefinition {
                            parent_type_name: _rt::string_lift(bytes0),
                            field_name: _rt::string_lift(bytes1),
                        },
                        result5,
                        _rt::string_lift(bytes6),
                    );
                    let ptr8 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec15 = result7;
                    let len15 = vec15.len();
                    let layout15 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec15.len() * 20,
                        4,
                    );
                    let result15 = if layout15.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout15).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout15);
                        }
                        ptr
                    } else {
                        { ::core::ptr::null_mut() }
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
                                    let super::super::super::super::component::grafbase::types::Error {
                                        extensions: extensions9,
                                        message: message9,
                                    } = e;
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
                                        { ::core::ptr::null_mut() }
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
                pub unsafe fn __post_return_authorize_edge_node_post_execution<T: Guest>(
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
                                    let bytes9 = _rt::Vec::from_raw_parts(
                                        l7.cast(),
                                        len9,
                                        len9,
                                    );
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::EdgeDefinition {
                            parent_type_name: _rt::string_lift(bytes0),
                            field_name: _rt::string_lift(bytes1),
                        },
                        result11,
                        _rt::string_lift(bytes12),
                    );
                    let ptr14 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec21 = result13;
                    let len21 = vec21.len();
                    let layout21 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec21.len() * 20,
                        4,
                    );
                    let result21 = if layout21.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout21).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout21);
                        }
                        ptr
                    } else {
                        { ::core::ptr::null_mut() }
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
                                    let super::super::super::super::component::grafbase::types::Error {
                                        extensions: extensions15,
                                        message: message15,
                                    } = e;
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
                                        { ::core::ptr::null_mut() }
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
                pub unsafe fn __post_return_authorize_edge_post_execution<T: Guest>(
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
                pub trait Guest {
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
                }
                #[doc(hidden)]
                macro_rules! __export_component_grafbase_authorization_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "component:grafbase/authorization#authorize-edge-pre-execution"]
                        unsafe extern "C" fn export_authorize_edge_pre_execution(arg0 :
                        i32, arg1 : * mut u8, arg2 : usize, arg3 : * mut u8, arg4 :
                        usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut u8, arg8 :
                        usize,) -> * mut u8 { $($path_to_types)*::
                        _export_authorize_edge_pre_execution_cabi::<$ty > (arg0, arg1,
                        arg2, arg3, arg4, arg5, arg6, arg7, arg8) } #[export_name =
                        "cabi_post_component:grafbase/authorization#authorize-edge-pre-execution"]
                        unsafe extern "C" fn
                        _post_return_authorize_edge_pre_execution(arg0 : * mut u8,) {
                        $($path_to_types)*::
                        __post_return_authorize_edge_pre_execution::<$ty > (arg0) }
                        #[export_name =
                        "component:grafbase/authorization#authorize-node-pre-execution"]
                        unsafe extern "C" fn export_authorize_node_pre_execution(arg0 :
                        i32, arg1 : * mut u8, arg2 : usize, arg3 : * mut u8, arg4 :
                        usize,) -> * mut u8 { $($path_to_types)*::
                        _export_authorize_node_pre_execution_cabi::<$ty > (arg0, arg1,
                        arg2, arg3, arg4) } #[export_name =
                        "cabi_post_component:grafbase/authorization#authorize-node-pre-execution"]
                        unsafe extern "C" fn
                        _post_return_authorize_node_pre_execution(arg0 : * mut u8,) {
                        $($path_to_types)*::
                        __post_return_authorize_node_pre_execution::<$ty > (arg0) }
                        #[export_name =
                        "component:grafbase/authorization#authorize-parent-edge-post-execution"]
                        unsafe extern "C" fn
                        export_authorize_parent_edge_post_execution(arg0 : i32, arg1 : *
                        mut u8, arg2 : usize, arg3 : * mut u8, arg4 : usize, arg5 : * mut
                        u8, arg6 : usize, arg7 : * mut u8, arg8 : usize,) -> * mut u8 {
                        $($path_to_types)*::
                        _export_authorize_parent_edge_post_execution_cabi::<$ty > (arg0,
                        arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) } #[export_name =
                        "cabi_post_component:grafbase/authorization#authorize-parent-edge-post-execution"]
                        unsafe extern "C" fn
                        _post_return_authorize_parent_edge_post_execution(arg0 : * mut
                        u8,) { $($path_to_types)*::
                        __post_return_authorize_parent_edge_post_execution::<$ty > (arg0)
                        } #[export_name =
                        "component:grafbase/authorization#authorize-edge-node-post-execution"]
                        unsafe extern "C" fn
                        export_authorize_edge_node_post_execution(arg0 : i32, arg1 : *
                        mut u8, arg2 : usize, arg3 : * mut u8, arg4 : usize, arg5 : * mut
                        u8, arg6 : usize, arg7 : * mut u8, arg8 : usize,) -> * mut u8 {
                        $($path_to_types)*::
                        _export_authorize_edge_node_post_execution_cabi::<$ty > (arg0,
                        arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) } #[export_name =
                        "cabi_post_component:grafbase/authorization#authorize-edge-node-post-execution"]
                        unsafe extern "C" fn
                        _post_return_authorize_edge_node_post_execution(arg0 : * mut u8,)
                        { $($path_to_types)*::
                        __post_return_authorize_edge_node_post_execution::<$ty > (arg0) }
                        #[export_name =
                        "component:grafbase/authorization#authorize-edge-post-execution"]
                        unsafe extern "C" fn export_authorize_edge_post_execution(arg0 :
                        i32, arg1 : * mut u8, arg2 : usize, arg3 : * mut u8, arg4 :
                        usize, arg5 : * mut u8, arg6 : usize, arg7 : * mut u8, arg8 :
                        usize,) -> * mut u8 { $($path_to_types)*::
                        _export_authorize_edge_post_execution_cabi::<$ty > (arg0, arg1,
                        arg2, arg3, arg4, arg5, arg6, arg7, arg8) } #[export_name =
                        "cabi_post_component:grafbase/authorization#authorize-edge-post-execution"]
                        unsafe extern "C" fn
                        _post_return_authorize_edge_post_execution(arg0 : * mut u8,) {
                        $($path_to_types)*::
                        __post_return_authorize_edge_post_execution::<$ty > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_grafbase_authorization_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 20]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 20],
                );
            }
            #[allow(dead_code, clippy::all)]
            pub mod responses {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type SharedContext = super::super::super::super::component::grafbase::types::SharedContext;
                pub type ExecutedOperation = super::super::super::super::component::grafbase::types::ExecutedOperation;
                pub type ExecutedSubgraphRequest = super::super::super::super::component::grafbase::types::ExecutedSubgraphRequest;
                pub type ExecutedHttpRequest = super::super::super::super::component::grafbase::types::ExecutedHttpRequest;
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
                            use super::super::super::super::component::grafbase::types::SubgraphRequestExecutionKind as V7;
                            let v7 = match l3 {
                                0 => V7::InternalServerError,
                                1 => V7::HookError,
                                2 => V7::RequestError,
                                3 => V7::RateLimited,
                                n => {
                                    debug_assert_eq!(n, 4, "invalid enum discriminant");
                                    let e7 = {
                                        let l4 = *base.add(8).cast::<i64>();
                                        let l5 = *base.add(16).cast::<i64>();
                                        let l6 = i32::from(*base.add(24).cast::<u16>());
                                        super::super::super::super::component::grafbase::types::SubgraphResponse {
                                            connection_time_ms: l4 as u64,
                                            response_time_ms: l5 as u64,
                                            status_code: l6 as u16,
                                        }
                                    };
                                    V7::Response(e7)
                                }
                            };
                            v7
                        };
                        result8.push(e8);
                    }
                    _rt::cabi_dealloc(base8, len8 * 32, 8);
                    let result9 = T::on_subgraph_response(
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::ExecutedSubgraphRequest {
                            subgraph_name: _rt::string_lift(bytes0),
                            method: _rt::string_lift(bytes1),
                            url: _rt::string_lift(bytes2),
                            executions: result8,
                            cache_status: super::super::super::super::component::grafbase::types::CacheStatus::_lift(
                                arg9 as u8,
                            ),
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
                pub unsafe fn __post_return_on_subgraph_response<T: Guest>(
                    arg0: *mut u8,
                ) {
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
                    use super::super::super::super::component::grafbase::types::GraphqlResponseStatus as V2;
                    let v2 = match arg9 {
                        0 => V2::Success,
                        1 => {
                            let e2 = super::super::super::super::component::grafbase::types::FieldError {
                                count: arg10 as u64,
                                data_is_null: _rt::bool_lift(arg11 as u8),
                            };
                            V2::FieldError(e2)
                        }
                        2 => {
                            let e2 = super::super::super::super::component::grafbase::types::RequestError {
                                count: arg10 as u64,
                            };
                            V2::RequestError(e2)
                        }
                        n => {
                            debug_assert_eq!(n, 3, "invalid enum discriminant");
                            V2::RefusedRequest
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::ExecutedOperation {
                            name: match arg1 {
                                0 => None,
                                1 => {
                                    let e = {
                                        let len0 = arg3;
                                        let bytes0 = _rt::Vec::from_raw_parts(
                                            arg2.cast(),
                                            len0,
                                            len0,
                                        );
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
                pub unsafe fn __post_return_on_operation_response<T: Guest>(
                    arg0: *mut u8,
                ) {
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
                        super::super::super::super::component::grafbase::types::SharedContext::from_handle(
                            arg0 as u32,
                        ),
                        super::super::super::super::component::grafbase::types::ExecutedHttpRequest {
                            method: _rt::string_lift(bytes0),
                            url: _rt::string_lift(bytes1),
                            status_code: arg5 as u16,
                            on_operation_response_outputs: result5,
                        },
                    );
                }
                pub trait Guest {
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
                    fn on_http_response(
                        context: SharedContext,
                        request: ExecutedHttpRequest,
                    );
                }
                #[doc(hidden)]
                macro_rules! __export_component_grafbase_responses_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "component:grafbase/responses#on-subgraph-response"] unsafe
                        extern "C" fn export_on_subgraph_response(arg0 : i32, arg1 : *
                        mut u8, arg2 : usize, arg3 : * mut u8, arg4 : usize, arg5 : * mut
                        u8, arg6 : usize, arg7 : * mut u8, arg8 : usize, arg9 : i32,
                        arg10 : i64, arg11 : i32,) -> * mut u8 { $($path_to_types)*::
                        _export_on_subgraph_response_cabi::<$ty > (arg0, arg1, arg2,
                        arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) }
                        #[export_name =
                        "cabi_post_component:grafbase/responses#on-subgraph-response"]
                        unsafe extern "C" fn _post_return_on_subgraph_response(arg0 : *
                        mut u8,) { $($path_to_types)*::
                        __post_return_on_subgraph_response::<$ty > (arg0) } #[export_name
                        = "component:grafbase/responses#on-operation-response"] unsafe
                        extern "C" fn export_on_operation_response(arg0 : i32, arg1 :
                        i32, arg2 : * mut u8, arg3 : usize, arg4 : * mut u8, arg5 :
                        usize, arg6 : i64, arg7 : i32, arg8 : i64, arg9 : i32, arg10 :
                        i64, arg11 : i32, arg12 : * mut u8, arg13 : usize,) -> * mut u8 {
                        $($path_to_types)*:: _export_on_operation_response_cabi::<$ty >
                        (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9,
                        arg10, arg11, arg12, arg13) } #[export_name =
                        "cabi_post_component:grafbase/responses#on-operation-response"]
                        unsafe extern "C" fn _post_return_on_operation_response(arg0 : *
                        mut u8,) { $($path_to_types)*::
                        __post_return_on_operation_response::<$ty > (arg0) }
                        #[export_name = "component:grafbase/responses#on-http-response"]
                        unsafe extern "C" fn export_on_http_response(arg0 : i32, arg1 : *
                        mut u8, arg2 : usize, arg3 : * mut u8, arg4 : usize, arg5 : i32,
                        arg6 : * mut u8, arg7 : usize,) { $($path_to_types)*::
                        _export_on_http_response_cabi::<$ty > (arg0, arg1, arg2, arg3,
                        arg4, arg5, arg6, arg7) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_grafbase_responses_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 8]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 8],
                );
            }
        }
    }
}
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
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub use alloc_crate::alloc;
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
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
        $($path_to_types_root)*::
        exports::component::grafbase::gateway_request::__export_component_grafbase_gateway_request_cabi!($ty
        with_types_in $($path_to_types_root)*::
        exports::component::grafbase::gateway_request); $($path_to_types_root)*::
        exports::component::grafbase::subgraph_request::__export_component_grafbase_subgraph_request_cabi!($ty
        with_types_in $($path_to_types_root)*::
        exports::component::grafbase::subgraph_request); $($path_to_types_root)*::
        exports::component::grafbase::authorization::__export_component_grafbase_authorization_cabi!($ty
        with_types_in $($path_to_types_root)*::
        exports::component::grafbase::authorization); $($path_to_types_root)*::
        exports::component::grafbase::responses::__export_component_grafbase_responses_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::component::grafbase::responses);
    };
}
#[doc(inline)]
pub(crate) use __export_hooks_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:hooks:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 2992] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xb4\x16\x01A\x02\x01\
A\x13\x01B:\x01m\x02\x14invalid-header-value\x13invalid-header-name\x04\0\x0chea\
der-error\x03\0\0\x01p}\x01q\x02\x0cchannel-full\x01\x02\0\x0echannel-closed\0\0\
\x04\0\x09log-error\x03\0\x03\x04\0\x07context\x03\x01\x04\0\x0eshared-context\x03\
\x01\x04\0\x07headers\x03\x01\x01r\x02\x10parent-type-names\x0afield-names\x04\0\
\x0fedge-definition\x03\0\x08\x01r\x01\x09type-names\x04\0\x0fnode-definition\x03\
\0\x0a\x01p\x02\x01r\x04\x06methods\x03urls\x0bstatus-code{\x1don-operation-resp\
onse-outputs\x0c\x04\0\x15executed-http-request\x03\0\x0d\x01r\x02\x05countw\x0c\
data-is-null\x7f\x04\0\x0bfield-error\x03\0\x0f\x01r\x01\x05countw\x04\0\x0drequ\
est-error\x03\0\x11\x01q\x04\x07success\0\0\x0bfield-error\x01\x10\0\x0drequest-\
error\x01\x12\0\x0frefused-request\0\0\x04\0\x17graphql-response-status\x03\0\x13\
\x01ks\x01r\x07\x04name\x15\x08documents\x13prepare-duration-msw\x0bcached-plan\x7f\
\x0bduration-msw\x06status\x14\x1con-subgraph-response-outputs\x0c\x04\0\x12exec\
uted-operation\x03\0\x16\x01r\x03\x12connection-time-msw\x10response-time-msw\x0b\
status-code{\x04\0\x11subgraph-response\x03\0\x18\x01m\x03\x03hit\x0bpartial-hit\
\x04miss\x04\0\x0ccache-status\x03\0\x1a\x01q\x05\x15internal-server-error\0\0\x0a\
hook-error\0\0\x0drequest-error\0\0\x0crate-limited\0\0\x08response\x01\x19\0\x04\
\0\x1fsubgraph-request-execution-kind\x03\0\x1c\x01p\x1d\x01r\x07\x0dsubgraph-na\
mes\x06methods\x03urls\x0aexecutions\x1e\x0ccache-status\x1b\x11total-duration-m\
sw\x0ahas-errors\x7f\x04\0\x19executed-subgraph-request\x03\0\x1f\x01o\x02ss\x01\
p!\x01r\x02\x0aextensions\"\x07messages\x04\0\x05error\x03\0#\x01h\x05\x01@\x02\x04\
self%\x04names\0\x15\x04\0\x13[method]context.get\x01&\x01@\x03\x04self%\x04name\
s\x05values\x01\0\x04\0\x13[method]context.set\x01'\x04\0\x16[method]context.del\
ete\x01&\x01h\x06\x01@\x02\x04self(\x04names\0\x15\x04\0\x1a[method]shared-conte\
xt.get\x01)\x01j\0\x01\x04\x01@\x02\x04self(\x04data\x02\0*\x04\0![method]shared\
-context.log-access\x01+\x01@\x01\x04self(\0s\x04\0\x1f[method]shared-context.tr\
ace-id\x01,\x01h\x07\x01@\x02\x04self-\x04names\0\x15\x04\0\x13[method]headers.g\
et\x01.\x01j\0\x01\x01\x01@\x03\x04self-\x04names\x05values\0/\x04\0\x13[method]\
headers.set\x010\x04\0\x16[method]headers.delete\x01.\x03\x01\x18component:grafb\
ase/types\x05\0\x02\x03\0\0\x07headers\x02\x03\0\0\x05error\x02\x03\0\0\x07conte\
xt\x01B\x0b\x02\x03\x02\x01\x01\x04\0\x07headers\x03\0\0\x02\x03\x02\x01\x02\x04\
\0\x05error\x03\0\x02\x02\x03\x02\x01\x03\x04\0\x07context\x03\0\x04\x01i\x05\x01\
i\x01\x01j\0\x01\x03\x01@\x02\x07context\x06\x07headers\x07\0\x08\x04\0\x12on-ga\
teway-request\x01\x09\x04\x01\"component:grafbase/gateway-request\x05\x04\x02\x03\
\0\0\x0eshared-context\x01B\x0b\x02\x03\x02\x01\x05\x04\0\x0eshared-context\x03\0\
\0\x02\x03\x02\x01\x01\x04\0\x07headers\x03\0\x02\x02\x03\x02\x01\x02\x04\0\x05e\
rror\x03\0\x04\x01i\x01\x01i\x03\x01j\0\x01\x05\x01@\x05\x07context\x06\x0dsubgr\
aph-names\x06methods\x03urls\x07headers\x07\0\x08\x04\0\x13on-subgraph-request\x01\
\x09\x04\x01#component:grafbase/subgraph-request\x05\x06\x02\x03\0\0\x0fedge-def\
inition\x02\x03\0\0\x0fnode-definition\x01B\x18\x02\x03\x02\x01\x02\x04\0\x05err\
or\x03\0\0\x02\x03\x02\x01\x05\x04\0\x0eshared-context\x03\0\x02\x02\x03\x02\x01\
\x07\x04\0\x0fedge-definition\x03\0\x04\x02\x03\x02\x01\x08\x04\0\x0fnode-defini\
tion\x03\0\x06\x01i\x03\x01j\0\x01\x01\x01@\x04\x07context\x08\x0adefinition\x05\
\x09argumentss\x08metadatas\0\x09\x04\0\x1cauthorize-edge-pre-execution\x01\x0a\x01\
@\x03\x07context\x08\x0adefinition\x07\x08metadatas\0\x09\x04\0\x1cauthorize-nod\
e-pre-execution\x01\x0b\x01ps\x01p\x09\x01@\x04\x07context\x08\x0adefinition\x05\
\x07parents\x0c\x08metadatas\0\x0d\x04\0$authorize-parent-edge-post-execution\x01\
\x0e\x01@\x04\x07context\x08\x0adefinition\x05\x05nodes\x0c\x08metadatas\0\x0d\x04\
\0\"authorize-edge-node-post-execution\x01\x0f\x01o\x02s\x0c\x01p\x10\x01@\x04\x07\
context\x08\x0adefinition\x05\x05edges\x11\x08metadatas\0\x0d\x04\0\x1dauthorize\
-edge-post-execution\x01\x12\x04\x01\x20component:grafbase/authorization\x05\x09\
\x02\x03\0\0\x12executed-operation\x02\x03\0\0\x19executed-subgraph-request\x02\x03\
\0\0\x15executed-http-request\x01B\x10\x02\x03\x02\x01\x05\x04\0\x0eshared-conte\
xt\x03\0\0\x02\x03\x02\x01\x0a\x04\0\x12executed-operation\x03\0\x02\x02\x03\x02\
\x01\x0b\x04\0\x19executed-subgraph-request\x03\0\x04\x02\x03\x02\x01\x0c\x04\0\x15\
executed-http-request\x03\0\x06\x01i\x01\x01p}\x01@\x02\x07context\x08\x07reques\
t\x05\0\x09\x04\0\x14on-subgraph-response\x01\x0a\x01@\x02\x07context\x08\x07req\
uest\x03\0\x09\x04\0\x15on-operation-response\x01\x0b\x01@\x02\x07context\x08\x07\
request\x07\x01\0\x04\0\x10on-http-response\x01\x0c\x04\x01\x1ccomponent:grafbas\
e/responses\x05\x0d\x04\x01\x18component:grafbase/hooks\x04\0\x0b\x0b\x01\0\x05h\
ooks\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.215.0\
\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
