/// Arguments passed to the `authorize_edge_node_post_execution` hook.
pub struct EdgeNodePostExecutionArguments {
    definition: crate::wit::EdgeDefinition,
    nodes: Vec<String>,
    metadata: String,
}

impl EdgeNodePostExecutionArguments {
    pub(crate) fn new(definition: crate::wit::EdgeDefinition, nodes: Vec<String>, metadata: String) -> Self {
        Self {
            definition,
            nodes,
            metadata,
        }
    }

    /// The name of the parent type of the edge.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type User {
    ///     id: Int!
    ///     name: String!
    /// }
    ///
    /// type Query {
    ///     users: [User!]! @authorized(node: "id")
    /// }
    /// ```
    ///
    /// And the query:
    ///
    /// ```graphql
    /// query {
    ///     users { name }
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
    /// type User {
    ///     id: Int!
    ///     name: String!
    /// }
    ///
    /// type Query {
    ///     users: [User!]! @authorized(node: "id")
    /// }
    /// ```
    ///
    /// And the query:
    ///
    /// ```graphql
    /// query {
    ///     users { name }
    /// }
    /// ```
    ///
    /// The field name is `users`.
    pub fn field_name(&self) -> &str {
        &self.definition.field_name
    }

    /// The nodes of the edge, serialized as a JSON objects.
    /// This method will deserialize the nodes into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type User {
    ///     id: Int!
    ///     name: String!
    /// }
    ///
    /// type Query {
    ///     users: [User!]! @authorized(node: "id")
    /// }
    /// ```
    ///
    /// And the query:
    ///
    /// ```graphql
    /// query {
    ///     users { name }
    /// }
    /// ```
    ///
    /// The query returns two entities:
    ///
    /// ```json
    /// [
    ///   {
    ///     "id": 1,
    ///     "name": "Alice"
    ///   },
    ///   {
    ///     "id": 2,
    ///     "name": "Bob"
    ///   }
    /// ]
    /// ```
    ///
    /// The arguments can be deserialized into a custom struct like:
    ///
    /// ```rust
    /// #[derive(serde::Deserialize)]
    /// struct User {
    ///    id: u64,
    /// }
    ///
    /// # fn foo(arguments: grafbase_hooks::EdgeNodePostExecutionArguments) -> Result<(), serde_json::Error> {
    /// let parents: Vec<User> = arguments.nodes()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The directive defines the `node` argument as `id`, so the hook gets an object of all
    /// ids of the returned users.
    pub fn nodes<'a, T>(&'a self) -> Result<Vec<T>, serde_json::Error>
    where
        T: serde::Deserialize<'a>,
    {
        self.nodes.iter().map(|parent| serde_json::from_str(parent)).collect()
    }

    /// The metadata passed to the `@authorized` directive. The metadata is
    /// serialized as a JSON object. This method will deserialize the metadata
    /// into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type User {
    ///     id: Int!
    ///     name: String!
    /// }
    ///
    /// type Query {
    ///     users: [User!]! @authorized(node: "id", metadata: { role: "admin" })
    /// }
    /// ```
    ///
    /// When executing a query like:
    ///
    /// ```graphql
    /// query {
    ///   users { name }
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
