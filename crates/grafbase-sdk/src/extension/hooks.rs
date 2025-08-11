use crate::{
    component::AnyExtension,
    host_io::{event_queue::EventQueue, http::StatusCode},
    types::{
        Configuration, Error, ErrorResponse, GatewayHeaders, Headers, HttpRequestParts, OnRequestOutput,
        OperationContext, RequestContext,
    },
};

/// The Hooks extension allows you to hook into an incoming request or an outgoing response.
///
/// You have mutable access to the headers, and information about the request or response
/// to decide whether to continue processing or not.
///
/// Keep in mind this is not meant for authentication purposes.
///
/// # Example
///
/// ```rust
/// use grafbase_sdk::{
///     HooksExtension,
///     types::{GatewayHeaders, Headers, Configuration, Error, ErrorResponse},
///     host_io::event_queue::EventQueue,
/// };
///
/// #[derive(HooksExtension)]
/// struct MyHooks {
///     config: Config,
/// }
///
/// #[derive(serde::Deserialize)]
/// struct Config {
///     // Define your configuration fields here. They are parsed from
///     // the grafbase.toml configuration.
///     something: String,
/// }
///
/// impl HooksExtension for MyHooks {
///     fn new(config: Configuration) -> Result<Self, Error> {
///         let config = config.deserialize()?;
///         Ok(Self { config })
///     }
///
///     #[allow(refining_impl_trait)]
///     fn on_request(&mut self, url: &str, method: http::Method, headers: &mut GatewayHeaders) -> Result<(), ErrorResponse> {
///         // Implement your request hook logic here.
///         Ok(())
///     }
///
///     fn on_response(
///         &mut self,
///         status: http::StatusCode,
///         headers: &mut Headers,
///         event_queue: EventQueue,
///     ) -> Result<(), Error> {
///         // Implement your response hook logic here.
///         Ok(())
///     }
/// }
/// ```
#[allow(unused_variables)]
pub trait HooksExtension: Sized + 'static {
    /// Creates a new instance of the extension. The [`Configuration`] will contain all the
    /// configuration defined in the `grafbase.toml` by the extension user in a serialized format.
    ///
    /// # Example
    ///
    /// The following TOML configuration:
    /// ```toml
    /// [extensions.my-hooks.config]
    /// my_custom_key = "value"
    /// ```
    ///
    /// can be easily deserialized with:
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{Configuration, Error};
    /// # fn dummy(config: Configuration) -> Result<(), Error> {
    /// #[derive(serde::Deserialize)]
    /// struct Config {
    ///     my_custom_key: String
    /// }
    ///
    /// let config: Config = config.deserialize()?;
    /// # Ok(())
    /// # }
    /// ```
    fn new(config: Configuration) -> Result<Self, Error>;

    /// Called immediately when a request is received, before entering the GraphQL engine.
    ///
    /// This hook can be used to modify the request headers before they are processed by the GraphQL engine, and provides a way to audit the headers, URL, and method before processing the operation.
    ///
    /// It can also be used to define the contract key to use for the contracts extension if you
    /// have any configured:
    ///
    /// ```rust
    /// # use grafbase_sdk::{host_io::http::Method, types::{ErrorResponse, GatewayHeaders, OnRequestOutput}};
    /// # struct MyContract;
    /// # impl MyContract {
    /// #[allow(refining_impl_trait)]
    /// fn on_request(&mut self, url: &str, method: Method, headers: &mut GatewayHeaders) -> Result<OnRequestOutput, ErrorResponse> {
    ///     Ok(OnRequestOutput::new().contract_key("my-contract-key"))
    /// }
    /// # }
    /// ```
    fn on_request(
        &mut self,
        url: &str,
        method: http::Method,
        headers: &mut GatewayHeaders,
    ) -> Result<impl IntoOnRequestOutput, ErrorResponse> {
        Ok(())
    }

    /// Called right before the response is sent back to the client.
    ///
    /// This hook can be used to modify the response headers before the response is sent back to the client.
    fn on_response(
        &mut self,
        ctx: &RequestContext,
        status: http::StatusCode,
        headers: &mut Headers,
        event_queue: EventQueue,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Called when a GraphQL subgraph request is made, allowing you to modify the request parts before they are sent to the subgraph.
    fn on_graphql_subgraph_request(
        &mut self,
        ctx: &OperationContext,
        subgraph_name: &str,
        parts: &mut HttpRequestParts,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Called when a virtual subgraph request is made through an extension, allowing you to modify the request headers before sending it to the extension.
    fn on_virtual_subgraph_request(
        &mut self,
        ctx: &OperationContext,
        subgraph_name: &str,
        headers: &mut Headers,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub trait IntoOnRequestOutput {
    fn into_on_request_output(self) -> OnRequestOutput;
}

impl IntoOnRequestOutput for OnRequestOutput {
    fn into_on_request_output(self) -> OnRequestOutput {
        self
    }
}

impl IntoOnRequestOutput for () {
    fn into_on_request_output(self) -> OnRequestOutput {
        OnRequestOutput::default()
    }
}

#[doc(hidden)]
pub fn register<T: HooksExtension>() {
    pub(super) struct Proxy<T: HooksExtension>(T);

    impl<T: HooksExtension> AnyExtension for Proxy<T> {
        fn on_request(
            &mut self,
            url: &str,
            method: http::Method,
            headers: &mut Headers,
        ) -> Result<OnRequestOutput, ErrorResponse> {
            self.0
                .on_request(url, method, headers)
                .map(|output| output.into_on_request_output())
        }

        fn on_response(
            &mut self,
            status: StatusCode,
            headers: &mut Headers,
            event_queue: EventQueue,
        ) -> Result<(), Error> {
            self.0.on_response(&RequestContext, status, headers, event_queue)
        }

        fn on_graphql_subgraph_request(
            &mut self,
            subgraph_name: &str,
            parts: &mut HttpRequestParts,
        ) -> Result<(), Error> {
            self.0
                .on_graphql_subgraph_request(&OperationContext, subgraph_name, parts)
        }

        fn on_virtual_subgraph_request(&mut self, subgraph_name: &str, headers: &mut Headers) -> Result<(), Error> {
            self.0
                .on_virtual_subgraph_request(&OperationContext, subgraph_name, headers)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as HooksExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
