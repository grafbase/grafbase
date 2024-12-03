/// Arguments passed to the `authorize_node_pre_execution` hook.
pub struct NodePreExecutionArguments {
    definition: crate::wit::NodeDefinition,
    metadata: String,
}

impl NodePreExecutionArguments {
    pub(crate) fn new(definition: crate::wit::NodeDefinition, metadata: String) -> Self {
        Self { definition, metadata }
    }

    /// The name of the node type.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type User @authorized {
    ///     id: Int!
    ///     name: String!
    /// }
    ///
    /// type Query {
    ///    user(id: ID!): User
    /// }
    /// ```
    ///
    /// The node type name is `User`.
    pub fn type_name(&self) -> &str {
        &self.definition.type_name
    }

    /// The metadata passed to the `@authorized` directive. The metadata is
    /// serialized as a JSON object. This method will deserialize the metadata
    /// into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type User @authorized(metadata: { role: "admin" }) {
    ///     id: Int!
    ///     name: String!
    /// }
    ///
    /// type Query {
    ///    user(id: ID!): User
    /// }
    /// ```
    ///
    /// When executing a query like:
    ///
    /// ```graphql
    /// query {
    ///   user(id: "123") { id }
    /// }
    /// ```
    ///
    /// The metadata is `{"role": "admin"}`.
    ///
    /// The metadata can be deserialized into a custom struct:
    ///
    /// ```rust
    /// #[derive(serde::Deserialize)]
    /// #[serde(untagged, rename = "snake_case")]
    /// enum Role {
    ///    Admin,
    ///    User,
    /// }
    ///
    /// #[derive(serde::Deserialize)]
    /// struct Metadata {
    ///    role: Role,
    /// }
    ///
    /// # fn foo(arguments: grafbase_hooks::EdgePreExecutionArguments) -> Result<(), serde_json::Error> {
    /// let arguments: Metadata = arguments.deserialize_metadata()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn deserialize_metadata<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.metadata)
    }
}
