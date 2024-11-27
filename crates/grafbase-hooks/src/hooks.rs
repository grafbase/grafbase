pub use crate::wit::{
    Context, EdgeDefinition, Error, ErrorResponse, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest,
    Guest, Headers, NodeDefinition, SharedContext,
};
use crate::Component;

pub(super) static mut HOOKS: Option<Box<dyn Hooks>> = None;

#[doc(hidden)]
pub fn hooks() -> &'static mut dyn Hooks {
    // SAFETY: a hook instance is single-threaded. This is created only once during initialization.
    // Every hook call happens in the same thread, there can ever be one caller at a time. Therefore
    // this is safe.
    #[allow(static_mut_refs)]
    unsafe {
        HOOKS.as_deref_mut().unwrap()
    }
}

impl Guest for Component {
    fn on_gateway_request(context: Context, headers: Headers) -> Result<(), ErrorResponse> {
        hooks().on_gateway_request(context, headers)
    }

    fn on_subgraph_request(
        context: SharedContext,
        subgraph_name: String,
        method: String,
        url: String,
        headers: Headers,
    ) -> Result<(), Error> {
        hooks().on_subgraph_request(context, subgraph_name, method, url, headers)
    }

    fn authorize_edge_pre_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        arguments: String,
        metadata: String,
    ) -> Result<(), Error> {
        hooks().authorize_edge_pre_execution(context, definition, arguments, metadata)
    }

    fn authorize_node_pre_execution(
        context: SharedContext,
        definition: NodeDefinition,
        metadata: String,
    ) -> Result<(), Error> {
        hooks().authorize_node_pre_execution(context, definition, metadata)
    }

    fn authorize_parent_edge_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        parents: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        hooks().authorize_parent_edge_post_execution(context, definition, parents, metadata)
    }

    fn authorize_edge_node_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        nodes: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        hooks().authorize_edge_node_post_execution(context, definition, nodes, metadata)
    }

    fn authorize_edge_post_execution(
        context: SharedContext,
        definition: EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        hooks().authorize_edge_post_execution(context, definition, edges, metadata)
    }

    fn on_subgraph_response(context: SharedContext, request: ExecutedSubgraphRequest) -> Vec<u8> {
        hooks().on_subgraph_response(context, request)
    }

    fn on_operation_response(context: SharedContext, request: ExecutedOperation) -> Vec<u8> {
        hooks().on_operation_response(context, request)
    }

    fn on_http_response(context: SharedContext, request: ExecutedHttpRequest) {
        hooks().on_http_response(context, request)
    }
}

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Missing grafbase_hooks macro on the Hooks implementation",
    label = "For this type",
    note = "Add #[grafbase_hooks] to the Hooks trait implementation for {Self}"
)]
pub trait HookImpls {
    fn hook_implementations(&self) -> u32;
}

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Missing register_hooks! macro invocation for the hooks implementation",
    label = "On this trait implementation",
    note = "Call register_hooks!({Self}) at the end of the file where the hooks implementation is defined"
)]
pub trait HookExports {}

/// Hooks are the main extension point for Grafbase. They allow you to intercept execution in various points of the request lifecycle.
///
/// To add a hook, you need to overload the default implementations of the hook functions in this trait and add the `#[grafbase_hooks]` attribute to the implementation.
#[allow(unused_variables)]
pub trait Hooks: HookImpls + HookExports {
    /// Initializes the hook. This is called once when a new hook instance is created.
    fn new() -> Self
    where
        Self: Sized;

    /// The gateway calls this hook just before authentication. You can use it
    /// to read and modify the request headers. You can store values in the context
    /// object for subsequent hooks to read.
    ///
    /// When the hook returns an error, processing stops and returns the error to the client.
    fn on_gateway_request(&mut self, context: Context, headers: Headers) -> Result<(), ErrorResponse> {
        todo!()
    }

    /// This hook runs before every subgraph request and after rate limiting. Use this hook to
    /// read and modify subgraph request headers. A returned error prevents the subgraph request.
    fn on_subgraph_request(
        &mut self,
        context: SharedContext,
        subgraph_name: String,
        method: String,
        url: String,
        headers: Headers,
    ) -> Result<(), Error> {
        todo!()
    }

    /// The request cycle calls this hook when the schema defines an authorization directive on
    /// an edge. The hook receives the edge's directive arguments, edge definition,
    /// and directive metadata.
    ///
    /// This hook runs before fetching any data.
    ///
    /// An error result stops request execution and returns the error to the user.
    /// The edge result becomes null for error responses.
    fn authorize_edge_pre_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        arguments: String,
        metadata: String,
    ) -> Result<(), Error> {
        todo!()
    }

    /// The gateway calls this hook during the request cycle when the schema defines an authorization directive for
    /// a node. The hook receives the node definition and directive metadata.
    ///
    /// This hook runs before any data fetching.
    ///
    /// An error result stops request execution and returns the error to the user.
    /// The edge value will be null for error responses.
    fn authorize_node_pre_execution(
        &mut self,
        context: SharedContext,
        definition: NodeDefinition,
        metadata: String,
    ) -> Result<(), Error> {
        todo!()
    }

    /// The request cycle runs this hook when the schema defines an authorization directive on
    /// an edge with the fields argument. The fields argument provides fields from the parent node.
    /// The hook receives parent type information and a list of data with the defined fields of
    /// the parent for every child that the parent query loads.
    ///
    /// This hook runs after data fetching completes.
    ///
    /// The hook returns one of the following:
    ///
    /// - A single-item list that defines the result for every child loaded from the edge
    /// - A multi-item list where each item defines child visibility
    ///
    /// Any other response causes the authorization hook to fail and prevents returning data to
    /// the user.
    ///
    /// A list item can be:
    ///
    /// - An empty Ok that returns edge data to the client
    /// - An error that denies edge access and propagates error data to response errors
    fn authorize_parent_edge_post_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        parents: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        todo!()
    }

    /// The request cycle runs this hook when the schema defines an authorization directive on
    /// an edge with the node argument, providing fields from the child node. This hook receives parent type information
    /// and a list of data with defined fields for every child the parent query loads.
    ///
    /// This hook runs after fetching the data.
    ///
    /// The result must be one of:
    ///
    /// - A single-item list that defines the result for every child loaded from the edge
    /// - A multi-item list where each item defines child visibility
    ///
    /// Any other response causes the authorization hook to fail and prevents returning data to
    /// the user.
    ///
    /// A list item can be:
    ///
    /// - An empty Ok that returns edge data to the client
    /// - An error that denies edge access and propagates error data to response errors
    fn authorize_edge_node_post_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        nodes: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        todo!()
    }

    /// The request cycle calls this hook when the schema defines an authorization directive on
    /// an edge with node and fields arguments, and provides fields from the child node. The hook receives
    /// parent type information and a list of data containing tuples of parent data and child data lists.
    ///
    /// The directive's fields argument defines the first part of the tuple and the node
    /// argument defines the second part.
    ///
    /// This hook runs after data fetching completes.
    ///
    /// The hook must return one of:
    ///
    /// - A single-item list that defines the result for every child loaded from the edge
    /// - A multi-item list where each item defines child visibility
    ///
    /// Any other response causes the authorization hook to fail and prevents returning data to
    /// the user.
    ///
    /// A list item can be:
    ///
    /// - An empty Ok that returns edge data to the client
    /// - An error that denies edge access and propagates error data to response errors
    fn authorize_edge_post_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        metadata: String,
    ) -> Vec<Result<(), Error>> {
        todo!()
    }

    /// This hook runs after the gateway requests a subgraph entity.
    /// It returns a byte vector that you can access in the `on_operation_response` hook.
    fn on_subgraph_response(&mut self, context: SharedContext, request: ExecutedSubgraphRequest) -> Vec<u8> {
        todo!()
    }

    /// The gateway calls this hook after it handles a request. The hook returns a list of bytes that
    /// the `on_http_response` hook can access.
    fn on_operation_response(&mut self, context: SharedContext, operation: ExecutedOperation) -> Vec<u8> {
        todo!()
    }

    /// The hook is called right before a response is sent to the user.
    fn on_http_response(&mut self, context: SharedContext, response: ExecutedHttpRequest) {
        todo!()
    }
}
