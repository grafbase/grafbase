/// Arguments passed to the `authorize_edge_post_execution` hook.
pub struct EdgePostExecutionArguments {
    definition: crate::wit::EdgeDefinition,
    edges: Vec<(String, Vec<String>)>,
    metadata: String,
}

impl EdgePostExecutionArguments {
    pub(crate) fn new(
        definition: crate::wit::EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        metadata: String,
    ) -> Self {
        Self {
            definition,
            edges,
            metadata,
        }
    }

    /// The name of the parent type of the edge.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Address {
    ///     street: String!
    /// }
    ///
    /// type User {
    ///     id: Int!
    ///     addresses: [Address!]! @authorized(fields: "id", node: "street")
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
    ///     users { addresses { street } }
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
    /// type Address {
    ///     street: String!
    /// }
    ///
    /// type User {
    ///     id: Int!
    ///     addresses: [Address!]! @authorized(fields: "id", node: "street")
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
    ///     users { addresses { street } }
    /// }
    /// ```
    ///
    /// The field name is `addresses`.
    pub fn field_name(&self) -> &str {
        &self.definition.field_name
    }

    /// The returned edges, serialized as a JSON objects. The first item of the tuple
    /// is a serialization of the fields defined in the `fields` argument and the second
    /// one is a serialization of the fields defined in the `node` argument.
    ///
    /// This method will deserialize the parent nodes into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Address {
    ///     street: String!
    /// }
    ///
    /// type User {
    ///     id: Int!
    ///     addresses: [Address!]! @authorized(fields: "id", node: "street")
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
    ///     users { addresses { street } }
    /// }
    /// ```
    ///
    /// The query returns two entities:
    ///
    /// ```json
    /// [
    ///   {
    ///     "id": 1,
    ///     "addresses": [{ "street": "Elm Street" }]
    ///   },
    ///   {
    ///     "id": 2,
    ///     "addresses": [{ "street": "Maple Street" }]
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
    /// #[derive(serde::Deserialize)]
    /// struct Node {
    ///    street: String,
    /// }
    ///
    /// # fn foo(arguments: grafbase_hooks::EdgePostExecutionArguments) -> Result<(), serde_json::Error> {
    /// let edges: Vec<(Parent, Vec<Node>)> = arguments.deserialize_edges()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The directive defines the `fields` argument as `id` and `node` argument as
    /// `street`, so the hook gets a vector of tuples where the first item is the
    /// parent fields with the fields defined in `fields` and the second one is
    /// the nodes with the fields defined in `node`.
    pub fn deserialize_edges<T, K>(&self) -> Result<Vec<(T, Vec<K>)>, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
        K: serde::de::DeserializeOwned,
    {
        self.edges
            .iter()
            .map(|(parent, nodes)| {
                let parent: T = serde_json::from_str(parent)?;

                let nodes: Vec<K> = nodes
                    .iter()
                    .map(|node| serde_json::from_str(node))
                    .collect::<Result<_, _>>()?;

                Ok((parent, nodes))
            })
            .collect()
    }

    /// The metadata passed to the `@authorized` directive. The metadata is
    /// serialized as a JSON object. This method will deserialize the metadata
    /// into either `serde_json::Value` or a custom struct.
    ///
    /// For the following GraphQL schema:
    ///
    /// ```graphql
    /// type Address {
    ///     street: String!
    /// }
    ///
    /// type User {
    ///     id: Int!
    ///     addresses: [Address!]! @authorized(
    ///         fields: "id",
    ///         node: "street",
    ///         metadata: { role: "admin" }
    ///     )
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
    ///   users { addresses { street } }
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
    /// # fn foo(arguments: grafbase_hooks::EdgePostExecutionArguments) -> Result<(), serde_json::Error> {
    /// let arguments: Metadata = arguments.deserialize_metadata()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn deserialize_metadata<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.metadata)
    }
}
