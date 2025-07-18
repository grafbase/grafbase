use schema::Schema;

use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
directive @join__unionMember(graph: join__Graph!, member: String!) on UNION

directive @join__implements(graph: join__Graph!, interface: String!) on OBJECT | INTERFACE

directive @join__graph(name: String!, url: String) on ENUM_VALUE

directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, overrideLabel: String) on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

directive @join__type(graph: join__Graph, key: join__FieldSet, extension: Boolean = false, resolvable: Boolean = true, isInterfaceObject: Boolean = false) on SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT

directive @join__owner(graph: join__Graph!) on OBJECT

directive @composite__lookup on FIELD_DEFINITION

directive @composite__require(field: composite__FieldSelectionMap!) on ARGUMENT_DEFINITION

directive @composite__is(field: composite__FieldSelectionMap!) on FIELD_DEFINITION | ARGUMENT_DEFINITION

scalar DateTime

scalar Duration

scalar join__FieldSet

scalar composite__FieldSelectionMap

type Call
  @join__type(graph: ZENDESK)
{
  associated_deal_ids: [ID!]! @inaccessible
  deals: [Deal!]! @composite__derive(graph: ZENDESK) @composite__is(graph: ZENDESK, field: "associated_deal_ids[{ id: . }]")
  deals2(ids: [ID!]! @composite__require(graph: ZENDESK, field: "associated_deal_ids")): [Deal!]! @extension__directive(graph: ZENDESK, extension: REST, name: "rest", arguments: {endpoint: "zendesk", method: GET, path: "/deals?ids={{#args.ids}}{{.}}{{^-last}},{{/-last}}{{/args.ids}}", selection: "[.items[] | .data | {\n  id,\n  name,\n  createdAt: .created_at\n}]"})
  duration: Duration!
  id: ID!
  summary: String!
}

type Deal
  @join__type(graph: ZENDESK, key: "id", resolvable: false)
{
  createdAt: DateTime!
  id: ID!
  name: String!
  orders(id: ID! @composite__require(graph: ZENDESK, field: "id")): [Order!]! @extension__directive(graph: ZENDESK, extension: REST, name: "rest", arguments: {endpoint: "zendesk", method: GET, path: "/orders?deal_id={{args.id}}", selection: "[.items[] | .data | {\n  id,\n  name,\n  deal_id,\n  createdAt: .created_at\n}]"})
}

type Order
  @join__type(graph: ZENDESK)
{
  createdAt: DateTime!
  deal: Deal! @composite__derive(graph: ZENDESK)
  deal_id: ID! @inaccessible
  id: ID!
  lineItems(id: ID! @composite__require(graph: ZENDESK, field: "id")): [LineItem!]! @extension__directive(graph: ZENDESK, extension: REST, name: "rest", arguments: {endpoint: "zendesk", method: GET, path: "/orders/{{args.id}}/line_items", selection: "[.items[] | .data | {\n  id,\n  product_id,\n  quantity\n}]"})
}

type LineItem
  @join__type(graph: ZENDESK)
{
  id: ID!
  product: Product! @composite__derive(graph: ZENDESK)
  product_id: ID! @inaccessible
}

type Product
  @join__type(graph: ZENDESK, key: "id", resolvable: false)
{
  description: String!
  id: ID!
  name: String!
}

type Query
{
  calls: [Call!]! @extension__directive(graph: ZENDESK, extension: REST, name: "rest", arguments: {endpoint: "zendesk", method: GET, path: "/calls", selection: "[.items[] | .data | {\n  id,\n  summary,\n  duration,\n  associated_deal_ids\n}]"}) @join__field(graph: ZENDESK)
  dealLookup(id: ID!): Deal! @inaccessible @composite__lookup(graph: ZENDESK) @extension__directive(graph: ZENDESK, extension: REST, name: "rest", arguments: {endpoint: "zendesk", method: GET, path: "/deals/{{args.id}}", selection: ".data | {\n  id,\n  name,\n  createdAt: .created_at\n}"}) @join__field(graph: ZENDESK)
  productsLookup(ids: [ID!]!): [Product!]! @inaccessible @composite__lookup(graph: ZENDESK) @extension__directive(graph: ZENDESK, extension: REST, name: "rest", arguments: {endpoint: "zendesk", method: GET, path: "/products?ids={{#args.ids}}{{.}}{{^-last}},{{/-last}}{{/args.ids}}", selection: "[.items[] | .data | {\n  id,\n  name,\n  description\n}]"}) @join__field(graph: ZENDESK)
}

enum join__Graph
{
  ZENDESK @join__graph(name: "zendesk")
}

enum extension__Link
{
  REST @extension__link(url: "file:///rest/build", schemaDirectives: [{graph: ZENDESK, name: "restEndpoint", arguments: {name: "zendesk", baseURL: "http://localhost:8080/v2", headers: [{name: "Accept", value: "application/json"}]}}])
}
"#;

const REST_SDL: &str = r#"
scalar UrlTemplate
scalar JqTemplate

"""
@restEndpoint directive enables defining named REST endpoints for the
schema. The directive can be used multiple times on a schema to
define different endpoints.

Example:
extend schema
  @restEndpoint(name: "weather", baseURL: "https://api.weather.com")
  @restEndpoint(name: "users", baseURL: "https://api.users.example.com")
"""
directive @restEndpoint(
  """
  A unique identifier for the REST endpoint
  """
  name: String!
  """
  The base URL for the REST API
  """
  baseURL: String!
  "Header to send to the endpoint"
  headers: [HTTPHeaderMapping!]
) repeatable on SCHEMA

input HTTPHeaderMapping {
  "Name of the HTTP header"
  name: String!
  """
  Static header value. It's a template that accepts a `config` parameter which
  represents the TOML config associated with this extension. So for example:
  `value: "Bearer: {{ config.token }}"`
  """
  value: String!
}

"""
@rest directive allows you to define RESTful API integrations for GraphQL
fields. This directive maps GraphQL fields to REST endpoints, enabling
seamless integration between your GraphQL schema and external REST APIs.

Example:
type Query {
  users: [User] @rest(
    endpoint: "users",
    method: GET,
    path: "/users",
  )
}
"""
directive @rest(
  """
  The name of the REST endpoint to use, as defined by @restEndpoint
  """
  endpoint: String!

  """
  The HTTP method to use for the request, such as GET, POST, etc.
  """
  method: HttpMethod!

  """
  The path template for the request, which can include
  variable substitutions from GraphQL arguments.
  This supports templating using GraphQL arguments: {{args.myArgument}}
  """
  path: UrlTemplate!

  """
  Specifies which fields from the GraphQL selection to include in the
  response.
  """
  selection: JqTemplate

  """
  Configuration for the request body, can include static values or
  selections from the GraphQL arguments
  """
  body: Body = { selection: ".args.input" }
) on FIELD_DEFINITION

scalar JSON

"""
Body input type defines how to construct the request body for REST
API calls. It allows for dynamic construction from GraphQL arguments
or static values.
"""
input Body {
  """
  Specifies which GraphQL arguments to include in the request body.
  Use "*" to include all arguments, or provide specific field names.
  """
  selection: JqTemplate

  """
  Static JSON content to include in the request body,
  which will be merged with any selected values.
  """
  static: JSON
}

"""
HttpMethod enum represents the standard HTTP methods supported
for REST API interactions.
"""
enum HttpMethod {
  """
  HTTP GET method for retrieving resources
  """
  GET
  """
  HTTP POST method for creating resources
  """
  POST
  """
  HTTP PUT method for replacing resources
  """
  PUT
  """
  HTTP DELETE method for removing resources
  """
  DELETE
  """
  HTTP HEAD method for retrieving headers only
  """
  HEAD
  """
  HTTP OPTIONS method for describing communication options
  """
  OPTIONS
  """
  HTTP CONNECT method for establishing tunnels
  """
  CONNECT
  """
  HTTP TRACE method for diagnostic testing
  """
  TRACE
  """
  HTTP PATCH method for partial modifications
  """
  PATCH
}
"#;

#[tokio::test]
async fn mix_of_look_derive_require() {
    let tmpdir = tempfile::tempdir().unwrap();
    let manifest = extension_catalog::Manifest {
        id: "rest-1.0.0".parse().unwrap(),
        r#type: extension_catalog::Type::Resolver(Default::default()),
        sdk_version: "0.0.0".parse().unwrap(),
        minimum_gateway_version: "0.0.0".parse().unwrap(),
        description: String::new(),
        sdl: Some(REST_SDL.into()),
        readme: None,
        homepage_url: None,
        repository_url: None,
        license: None,
        permissions: Default::default(),
        legacy_event_filter: Default::default(),
    };

    std::fs::write(
        tmpdir.path().join("manifest.json"),
        serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
    )
    .unwrap();

    let mut catalog = extension_catalog::ExtensionCatalog::default();
    let wasm_path = tmpdir.path().join("extension.wasm");
    std::fs::write(&wasm_path, b"wasm").unwrap();
    catalog.push(extension_catalog::Extension {
        config_key: String::new(),
        manifest,
        wasm_path,
    });

    let schema = Schema::builder(&SCHEMA.replace(
        "file:///rest/build",
        url::Url::from_file_path(tmpdir.path()).unwrap().as_str(),
    ))
    .extensions(None, &catalog)
    .build()
    .await
    .unwrap();

    // Extension resolver can be placed on arbitrary fields, its presence indicates that it must be
    // used to resolve the field. We're taking this into account when computing the providable
    // fields. However this logic was initially only applied for nested fields. Not the root
    // fields, which isn't necessary except for lookup. While building the schema we don't apply
    // the same, complex, logic. Everything that isn't part of the key will be assigned a lookup
    // resolver. So it's during the query planning that we need to check whether a field is
    // actually providable or not.
    assert_solving_snapshots!(
        "mix_of_look_derive_require",
        schema,
        r#"
        query Calls {
          calls {
            deals {
              orders {
                createdAt
                lineItems {
                  product {
                    name
                    description
                  }
                  id
                }
                id
              }
              createdAt
            }
          }
        }
        "#
    );
}
