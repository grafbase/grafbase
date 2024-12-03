/// Arguments passed to the `authorize_parent_edge_post_execution` hook.
pub struct ParentEdgePostExecutionArguments {
    definition: crate::wit::EdgeDefinition,
    parents: Vec<String>,
    metadata: String,
}

impl ParentEdgePostExecutionArguments {
    pub(crate) fn new(definition: crate::wit::EdgeDefinition, parents: Vec<String>, metadata: String) -> Self {
        Self {
            definition,
            parents,
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
    ///     name: String! @authorized(fields: "id")
    /// }
    ///
    /// type Query {
    ///     users: [User!]!
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
    /// The parent type name is `User`.
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
    ///     name: String! @authorized(fields: "id")
    /// }
    ///
    /// type Query {
    ///     users: [User!]!
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
    /// The field name is `name`.
    pub fn field_name(&self) -> &str {
        &self.definition.field_name
    }

    /// The parent nodes of the edge. The parent nodes are serialized as a JSON objects.
    /// This method will deserialize the parent nodes into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type User {
    ///     id: Int!
    ///     name: String! @authorized(fields: "id")
    /// }
    ///
    /// type Query {
    ///     users: [User!]!
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
    /// struct Parent {
    ///    id: u64,
    /// }
    ///
    /// # fn foo(arguments: grafbase_hooks::ParentEdgePostExecutionArguments) -> Result<(), serde_json::Error> {
    /// let parents: Vec<Parent> = arguments.deserialize_parents()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The directive defines the `fields` argument as `id`, so the hook gets an object of all
    /// ids of the returned users.
    pub fn deserialize_parents<T: serde::de::DeserializeOwned>(&self) -> Result<Vec<T>, serde_json::Error> {
        self.parents.iter().map(|parent| serde_json::from_str(parent)).collect()
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
    ///     name: String! @authorized(fields: "id", metadata: { role: "admin" })
    /// }
    ///
    /// type Query {
    ///     users: [User!]!
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
    /// # fn foo(arguments: grafbase_hooks::ParentEdgePostExecutionArguments) -> Result<(), serde_json::Error> {
    /// let arguments: Metadata = arguments.deserialize_metadata()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn deserialize_metadata<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.metadata)
    }
}
