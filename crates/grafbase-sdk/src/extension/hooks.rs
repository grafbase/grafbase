use crate::{
    component::AnyExtension,
    host_io::{
        event_queue::EventQueue,
        http::{Method, StatusCode},
    },
    types::{Configuration, Error, ErrorResponse, GatewayHeaders, OnRequestOutput},
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
///     types::{GatewayHeaders, Configuration, Error, ErrorResponse},
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
///     fn on_request(&mut self, url: &str, method: http::Method, headers: &mut GatewayHeaders) -> Result<(), ErrorResponse> {
///         // Implement your request hook logic here.
///         Ok(())
///     }
///
///     fn on_response(
///         &mut self,
///         status: http::StatusCode,
///         headers: &mut GatewayHeaders,
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
    fn on_request(
        &mut self,
        url: &str,
        method: http::Method,
        headers: &mut GatewayHeaders,
    ) -> Result<impl IntoOnRequestOutput, ErrorResponse>;

    /// Called right before the response is sent back to the client.
    ///
    /// This hook can be used to modify the response headers before the response is sent back to the client.
    fn on_response(
        &mut self,
        status: http::StatusCode,
        headers: &mut GatewayHeaders,
        event_queue: EventQueue,
    ) -> Result<(), Error>;
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
            method: Method,
            headers: &mut GatewayHeaders,
        ) -> Result<OnRequestOutput, ErrorResponse> {
            HooksExtension::on_request(&mut self.0, url, method, headers)
                .map(IntoOnRequestOutput::into_on_request_output)
        }

        fn on_response(
            &mut self,
            status: StatusCode,
            headers: &mut GatewayHeaders,
            event_queue: EventQueue,
        ) -> Result<(), Error> {
            HooksExtension::on_response(&mut self.0, status, headers, event_queue)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as HooksExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
