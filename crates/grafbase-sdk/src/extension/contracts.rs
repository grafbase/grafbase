use crate::{
    component::AnyExtension,
    types::{Configuration, Contract, ContractDirective, Error, GraphqlSubgraph},
};

/// The Contracts extension allows you to control which part of the schema will be exposed to
/// clients for GraphQL queries, introspection and also the MCP endpoint if active.
///
/// Contracts are built and cached for a particular key. This can be statically defined the
/// `grafbase.toml` file:
///
/// ```toml
/// [graph]
/// contract_key = "<key>"
/// ```
///
/// Or dynamically provided by the `on_request` hook:
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
///
/// In addition to the key, the extension will receive a list of all the directives defined by said
/// extension and the list of GraphQL subgraphs. For each directive it must specify whether the
/// decorated element is part of the exposed API or not. If not, they're treated as if
/// `@inaccessible` was applied on them.
///
/// # Example
///
/// You can initialize a new resolver extension with the Grafbase CLI:
///
/// ```bash
/// grafbase extension init --type contracts my-contracts
/// ```
///
/// ```rust
/// use grafbase_sdk::{
///     ContractsExtension,
///     types::{Configuration, Error, Contract, ContractDirective, GraphqlSubgraph},
/// };
///
/// #[derive(ContractsExtension)]
/// struct MyContracts;
///
/// impl ContractsExtension for MyContracts {
///     fn new(config: Configuration) -> Result<Self, Error> {
///         Ok(Self)
///     }
///
///     fn construct(
///         &mut self,
///         key: String,
///         directives: Vec<ContractDirective<'_>>,
///         subgraphs: Vec<GraphqlSubgraph>,
///     ) -> Result<Contract, Error> {
///         Ok(Contract::new(&directives, true))
///     }
/// }
/// ```
///
/// I
pub trait ContractsExtension: Sized + 'static {
    /// Creates a new instance of the extension. The [`Configuration`] will contain all the
    /// configuration defined in the `grafbase.toml` by the extension user in a serialized format.
    ///
    /// # Example
    ///
    /// The following TOML configuration:
    /// ```toml
    /// [extensions.my-contracts.config]
    /// my_custom_key = "value"
    /// ```
    ///
    /// can be easily deserialized with:
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{Configuration, Error};
    /// # fn dummy(config: Configuration) -> Result<(), Error> {
    /// #[derive(Default, serde::Deserialize)]
    /// #[serde(default, deny_unknown_fields)]
    /// struct Config {
    ///     my_custom_key: Option<String>
    /// }
    ///
    /// let config: Config = config.deserialize()?;
    /// # Ok(())
    /// # }
    /// ```
    fn new(config: Configuration) -> Result<Self, Error>;

    /// Create the contract based on the provided key. The contract specifies whether the elements
    /// decorated by directives are part of the exposed API or not. Furthermore it's possible to
    /// modify the GraphQL subgraphs for this contract.
    fn construct(
        &mut self,
        key: String,
        directives: Vec<ContractDirective<'_>>,
        subgraphs: Vec<GraphqlSubgraph>,
    ) -> Result<Contract, Error>;
}

#[doc(hidden)]
pub fn register<T: ContractsExtension>() {
    pub(super) struct Proxy<T: ContractsExtension>(T);

    impl<T: ContractsExtension> AnyExtension for Proxy<T> {
        fn construct(
            &mut self,
            key: String,
            directives: Vec<ContractDirective<'_>>,
            subgraphs: Vec<GraphqlSubgraph>,
        ) -> Result<Contract, Error> {
            ContractsExtension::construct(&mut self.0, key, directives, subgraphs)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as ContractsExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
