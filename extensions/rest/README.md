# REST Extension

This is a REST extension for the Grafbase Gateway. It allows you to define REST endpoints and map them to GraphQL fields. It provides two directives:

- `@restEndpoint`, which you can repeat on the schema, defines a REST endpoint.
- `@rest`, defined on a field, specifies which endpoint the field uses, which path and method it uses and how it selects data from the REST response.

## Installing

Build this extension manually and copy the artifacts to a location where the gateway can find them until we complete the Grafbase Extension Registry.

```bash
grafbase extension build
```

The `build` directory contains the resulting wasm component and manifest file.

```bash
build/
├── extension.wasm
└── manifest.json
```

In your gateway configuration, you can now load the extension from the `build` directory.

```toml
[extensions.rest]
path = "/path/to/build"
```

## Configuration

This extension acts as a field resolver for the Grafbase Gateway. Use it as a subgraph to provide a REST translator to GraphQL for the gateway.

The extension provides two directives:

- `@restEndpoint`, which you can repeat on the schema, defines a REST endpoint.
- `@rest`, defined on a field, specifies which endpoint the field uses, which path and method it uses and how it selects data from the REST response.

Define your REST endpoint in your subgraph definition:

```graphql
extend schema
  @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key", "@shareable"])
  @link(url: "file:///absolute/path/to/extension/build", import: ["@restEndpoint", "@rest"])

@restEndpoint(
  name: "restCountries",
  baseURL: "https://restcountries.com/v3.1"
)
```

The `@restEndpoint` takes a unique name per subgraph, which you must refer to in the corresponding `@rest` directives, and a `baseURL`.

The `@rest` directive goes to a field directive:

```graphql
type Country {
  name: String!
}

type Query {
  listAllCountries: [Country!]! @rest(
    endpoint: "restCountries",
    http: {
      method: GET,
      path: "/all"
    },
    selection: "[.[] | { name: .name.official }]"
  )
}
```

The `endpoint` argument must match one `@restEndpoint` definition in the same subgraph. The `http` arguments define the method and a path for this endpoint, and the `selection` defines how the data transforms from the REST endpoint to a GraphQL subgraph response.

In our example we query countries from a public endpoint, which returns JSON. If we curl the API:

```bash
curl https://restcountries.com/v3.1/all |jq |head -n100
```

Our data looks something like this:

```json
[
  {
    "name": {
      "common": "South Georgia",
      "official": "South Georgia and the South Sandwich Islands",
      "nativeName": {
        "eng": {
          "official": "South Georgia and the South Sandwich Islands",
          "common": "South Georgia"
        }
      }
    },
    ...
  }
}
```

Now, our GraphQL type for `Country` has only one field: the name. The `selection` argument in the `@rest` directive supports [jq filters](https://jqlang.org/manual/). First try the selection filter in the terminal with curl and jq. Convert the API endpoint from what we see above to the following, which should fit our GraphQL type definition:

```graphql
type Country {
  name: String!
}
```

The extension JSON output should have objects with the `name` key and the values must be strings. No nulls are allowed.

Run the following curl command:

```bash
curl https://restcountries.com/v3.1/all |jq "[.[] | { name: .name.official }]"
```

You'll get output:

```json
[
  {
    "name": "South Georgia and the South Sandwich Islands"
  },
  {
    "name": "Grenada"
  },
  ...
]
```

Save the subgraph schema we just defined into a file, and publish it to the Grafbase platform

```bash
grafbase publish --name countries -m init my-org/my-federated-graph
```

You can omit the `--url` parameter from a subgraph that only acts as a virtual graph for an extension.

## Request Body

Use the `body` argument to send data to the REST endpoint. The `body` argument accepts a JSON object or a selection that maps data from the input arguments.

To send dynamic data from the input arguments, add a selection to the body. The extension looks for a body in an argument named `input`. Use this name to follow the expected convention:

```graphql
type Mutation {
  createCountry(input: Country!): Country! @rest(
    endpoint: "restCountries",
    http: {
      method: POST,
      path: "/create"
    },
    selection: "{ name: .name.official }"
  )
}
```

You can also use static data in the body:

```graphql
type Mutation {
  createCountry: Country! @rest(
    endpoint: "restCountries",
    http: {
      method: POST,
      path: "/create"
    },
    body: { static: { name: "Georgia" } },
    selection: "{ name: .name.official }"
  )
}
```

The extension checks static data first, then searches for a body in an argument named `input`.

## Arguments

The path argument is used to specify the path to the REST endpoint. You can use the input arguments to construct the path:

```graphql
type Mutation {
  getCountry(id: Int!): Country @rest(
    endpoint: "restCountries",
    http: {
      method: GET,
      path: "/fetch/{{ args.id }}"
    },
    selection: "{ name: .name.official }"
  )
}
```

The extension will generate the path based on the `id` argument.
