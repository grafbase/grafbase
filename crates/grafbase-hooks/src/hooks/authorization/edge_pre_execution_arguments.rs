/// Arguments passed to the `authorize_edge_pre_execution` hook.
pub struct EdgePreExecutionArguments {
    definition: crate::wit::EdgeDefinition,
    arguments: String,
    metadata: String,
}

impl EdgePreExecutionArguments {
    pub(crate) fn new(definition: crate::wit::EdgeDefinition, arguments: String, metadata: String) -> Self {
        Self {
            definition,
            arguments,
            metadata,
        }
    }

    /// The name of the parent type of the edge.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Query {
    ///     user(id: ID!): User @authorized(arguments: "id")
    /// }
    /// ```
    ///
    /// The parent type name is `Query`.
    pub fn parent_type_name(&self) -> &str {
        &self.definition.parent_type_name
    }

    /// The name of the authorized edge.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Query {
    ///     user(id: ID!): User @authorized(arguments: "id")
    /// }
    /// ```
    ///
    /// The field name is `user`.
    pub fn field_name(&self) -> &str {
        &self.definition.field_name
    }

    /// The arguments passed to the `@authorized` directive. The arguments are
    /// serialized as a JSON object. This method will deserialize the arguments
    /// into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Query {
    ///     user(id: ID!): User @authorized(arguments: "id")
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
    /// The arguments are `{"id": "123"}`.
    ///
    /// The arguments can be deserialized into a custom struct like:
    ///
    /// ```rust
    /// #[derive(serde::Deserialize)]
    /// struct Arguments {
    ///    id: String,
    /// }
    ///
    /// # fn foo(arguments: grafbase_hooks::EdgePreExecutionArguments) -> Result<(), serde_json::Error> {
    /// let arguments: Arguments = arguments.arguments()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn arguments<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where
        T: serde::Deserialize<'a>,
    {
        serde_json::from_str(&self.arguments)
    }

    /// The metadata passed to the `@authorized` directive. The metadata is
    /// serialized as a JSON object. This method will deserialize the metadata
    /// into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Query {
    ///     user(id: ID!): User @authorized(arguments: "id", metadata: { role: "admin" })
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
    /// The metadata can be deserialized into a custom struct like:
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
    /// let arguments: Metadata = arguments.metadata()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn metadata<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where
        T: serde::Deserialize<'a>,
    {
        serde_json::from_str(&self.metadata)
    }
}
