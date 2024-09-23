<p align="center">
  <p align="center">⚠️ Grafbase SDK is now deprecated. Please see  <a href="https://grafbase.com/blog/sunsetting-standalone-graphs">Sunsetting Standalone Graphs</a> for more details ⚠️</p>
</p>

<p align="center">
  <a href="https://grafbase.com">
    <img src="https://grafbase.com/images/other/grafbase-logo-circle.png" height="96">
    <h3 align="center">Grafbase SDK</h3>
  </a>
</p>

<p align="center">
  Grafbase as configuration
</p>

<p align="center">
  <a href="/templates"><strong>Templates</strong></a> ·
  <a href="https://grafbase.com/docs"><strong>Docs</strong></a> ·
  <a href="https://grafbase.com/cli"><strong>CLI</strong></a> ·
  <a href="https://grafbase.com/community"><strong>Community</strong></a> ·
  <a href="https://grafbase.com/changelog"><strong>Changelog</strong></a>
</p>

<br/>

## Get Started

Adding to an existing project:

```bash
npm install @grafbase/sdk --save-dev
```

Initializing a new project

```bash
grafbase init --config-format typescript
```

## Example

The configuration should define the schema, exporting the config as `default`:

```typescript
// g is a schema generator, config the final object to return
import { graph, config } from '@grafbase/sdk'

const g = graph.Standalone()

// types are generated with the `type` method,
// followed by the name and fields.
const profile = g.type('Profile', {
  address: g.string()
})

// finally we export the default config
export default config({
  graph: g
})
```

When `grafbase dev` finds the above config from `$PROJECT/grafbase/grafbase.config.ts`, it genereates the SDL to `$PROJECT/.grafbase/generated/schema/schema.graphql`:

```graphql
type Profile {
  address: String!
}

type User @model {
  name: String!
  age: Int
  profile: Profile
  parent: User
}
```

The above SDL is now used when starting the dev.

## Federated Graphs

A federated graph can be defined as follows:

```typescript
import { graph, config } from '@grafbase/sdk'

export default config({
  graph: g.Federated()
})
```

## Types

Types are generated with the `type` method:

```typescript
g.type('Profile', {
  address: g.string()
})
```

## Enums

Enums can be generated with the `enum` method:

```typescript
g.enum('Fruits', ['Apples', 'Oranges'])
```

An enum can be used as a field type with the `enumRef` method:

```typescript
const e = g.enum('Fruits', ['Apples', 'Oranges'])

g.type('User', {
  favoriteFruit: g.enumRef(e)
})
```

## Queries and Mutations

Queries are generated with the `query` method, mutations with the `mutation` method:

```typescript
g.query('greet', {
  args: { name: g.string().optional() },
  returns: g.string(),
  resolver: 'hello'
})

const input = g.input('CheckoutSessionInput', { name: g.string() })
const output = g.type('CheckoutSessionOutput', { successful: g.boolean() })

g.mutation('checkout', {
  args: { input: g.inputRef(input) },
  returns: g.ref(output),
  resolver: 'checkout'
})
```

## Unions

Unions can be done using the `union` method:

```typescript
const user = g.type('User', {
  name: g.string(),
  age: g.int().optional()
})

const address = g.type('Address', {
  street: g.string().optional()
})

g.union('UserOrAddress', { user, address })
```

## Interfaces

Interfaces can be generated using the `interface` method, and a type can be extended with an interface:

```typescript
const produce = g.interface('Produce', {
  name: g.string(),
  quantity: g.int(),
  price: g.float(),
  nutrients: g.string().optional().list().optional()
})

g.type('Fruit', {
  isSeedless: g.boolean().optional(),
  ripenessIndicators: g.string().optional().list().optional()
}).implements(produce)
```

Notice how one doesn't need to type the fields to the type: they are inferred to the final SDL from the interface definition.

## Field generation

Fields are generated from the `g` object:

- ID: `g.id()`
- String: `g.string()`
- Int: `g.int()`
- Float: `g.float()`
- Boolean: `g.boolean()`
- Date: `g.date()`
- DateTime: `g.datetime()`
- Email: `g.email()`
- IPAddress: `g.ipAddress()`
- Timestamp: `g.timestamp()`
- URL: `g.url()`
- JSON: `g.json()`
- PhoneNumber: `g.phoneNumber()`

## Enum fields

```typescript
// first greate an enum
const fruits = g.enumType('Fruits', ['Apples', 'Oranges'])

// then use it e.g. in a model
g.model('User', {
  favoriteFruit: g.enum(fruits)
})
```

## Reference fields

Referencing a type is with the `ref` method:

```typescript
const profile = g.type('Profile', {
  address: g.string()
})

g.model('User', {
  profile: g.ref(profile)
})
```

## Optional fields

By default the generated fields are _required_. To make them optional is with the `optional` method:

```typescript
const user = g.type('User', {
  posts: g.string().optional()
})
```

## List fields

List fields can be done with the `list` method:

```typescript
const user = g.type('User', {
  names: g.string().list()
})
```

By default, the list or list items are _required_. To make the items nullable, call the `optional` method to the base type:

```typescript
const user = g.type('User', {
  names: g.string().optional().list()
})
```

To make the list itself optional, call the `optional` method to the list type:

```typescript
const user = g.type('User', {
  names: g.string().list().optional()
})
```

## Unique

Unique field can be defined to certain types of fields with the `unique` method:

```typescript
const user = g.type('User', {
  name: g.string().unique()
})
```

Additional unique scope can be given as a parameter:

```typescript
const user = g.type('User', {
  name: g.string().unique(['email']),
  email: g.string()
})
```

## Length limit

Certain field types can have a limited length:

```typescript
const user = g.type('User', {
  name: g.string().length({ min: 1, max: 255 })
})
```

## Connectors

Connectors are created through the connector interface:

```typescript
import { connector } from '../../src/index'
```

### OpenAPI

The OpenAPI connector can be created with the `OpenAPI` method:

```typescript
const openai = connector.OpenAPI("OpenAI",
  schema:
    'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml'
})

const stripe = connector.OpenAPI("Stripe", {
  schema:
    'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
  headers: (headers) => {
    // used in client and introspection requests
    headers.set('Authorization', `Bearer ${g.env('STRIPE_API_KEY')}`)
    // used only in introspection requests
    headers.introspection('foo', 'bar')
    // forward headers from requests to datasource
    headers.set('x-api-key', { forward: 'x-api-key' })
  }
})
```

Connectors can be added to the schema using `g.datasource()`, including an optional `namespace`:

```typescript
g.datasource(stripe)
g.datasource(openai)
```

### GraphQL

The GraphQL connector can be created with the `GraphQL` method:

```typescript
const contentful = connector.GraphQL('Contentful', {
  url: g.env('CONTENTFUL_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', `Bearer ${g.env('CONTENTFUL_API_KEY')}`)
    headers.set('Method', 'POST')
    headers.set('x-api-key', { forward: 'x-api-key' })
  }
})

const github = connector.GraphQL('GitHub', {
  url: 'https://api.github.com/graphql'
})
```

Connectors can be added to the schema using `g.datasource()`, including an optional `namespace`:

```typescript
g.datasource(contentful)
g.datasource(github)
```

### MongoDB

The MongoDB connector can be created with the `MongoDB` method:

```typescript
const mongodb = connector.MongoDB('MongoDB', {
  url: 'https://data.mongodb-api.com/app/data-test/endpoint/data/v1',
  apiKey: 'SOME_KEY',
  dataSource: 'data',
  database: 'tables'
})

// Models must be added manually for this connector.
mongodb.model('User', { field: g.string() }).collection('users')

g.datasource(mongodb)
```

### Authentication

Auth providers can be created through the `auth` object.

```typescript
import { auth } from '@grafbase/sdk'
```

#### OpenID

Required fields:

- `issuer`

Optional fields:

- `clientId`
- `groupsClaim`

```typescript
// first create the provider
const clerk = auth.OpenIDConnect({
  issuer: g.env('ISSUER_URL')
})

// add it to the config with the rules
const cfg = config({
  schema: g,
  auth: {
    providers: [clerk],
    rules: (rules) => {
      rules.private()
    }
  }
})
```

#### JWT

Required fields:

- `issuer`
- `secret`

Optional fields:

- `clientId`
- `groupsClaim`

```typescript
const derp = auth.JWT({
  issuer: g.env('ISSUER_URL'),
  secret: g.env('JWT_SECRET')
})
```

## JWKS

Required fields:

- `issuer`

Optional fields:

- `clientId`
- `groupsClaim`
- `jwksEndpoint`

A JWKS provider has to define _either_ `issuer` or `jwksEndpoint`

```typescript
const derp = auth.JWKS({
  issuer: g.env('ISSUER_URL')
})
```

## Authorizer

Required fields:

- `name`

```typescript
const authorizer = auth.Authorizer({
  name: 'custom-auth'
})
```

The name maps the name of the file including a custom authentication function. For this example, there has to be a file implementing the authentication function in `grafbase/auth/custom-auth.js`.

## Rule Definitions

Everywhere where one can define authentication rules, it happens through a lambda with a rules builder.

```typescript
{
  rules: (rules) => {
    rules.private().read()
    rules.groups(['admin', 'root']).delete()
  }
}
```

### Global Rules

Global rules are defined through the auth definition in the configuration.

```typescript
const clerk = auth.OpenIDConnect({
  issuer: g.env('ISSUER_URL')
})

const cfg = config({
  schema: g,
  auth: {
    providers: [clerk],
    rules: (rules) => {
      rules.private()
    }
  }
})
```

## Caching

Caching can be defined globally, per type or per field.

```ts
config({
  schema: g,
  cache: {
    rules: [
      {
        types: 'Query',
        maxAge: 60
      },
      {
        types: ['GitHub', 'Strava'],
        maxAge: 60,
        staleWhileRevalidate: 60
      },
      {
        types: [{ name: 'Query' }, { name: 'GitHub', fields: ['name'] }],
        maxAge: 60
      }
    ]
  }
})

g.model('User', {
  name: g.string().optional()
}).cache({
  maxAge: 60,
  staleWhileRevalidate: 60,
  mutationInvalidation: 'entity'
})

g.type('User', {
  name: g.string().optional()
}).cache({
  maxAge: 60,
  staleWhileRevalidate: 60,
  mutationInvalidation: 'type'
})

g.model('User', {
  name: g.string().cache({ maxAge: 60, staleWhileRevalidate: 60 })
})
```

### Extending Types

Types can be extended with extra queries, handled with resolvers.

To extend a type that is defined in the Grafbase schema, define the type first and extend it by using the type as the parameter:

```ts
const user = g.type('User', {
  name: g.string()
})

g.extend(user, {
  myField: {
    args: { myArg: g.string() },
    returns: g.string(),
    resolver: 'file'
  }
})
```

Sometimes a type is not defined directly in the schema, but instead introspected from an external connector. In these cases passing a string as the first argument allows extending the type with custom queries. Keep in mind that in these cases it is not validated in compile-time if the type exist.

```ts
g.extend('StripeCustomer', {
  myField: {
    args: { myArg: g.string() },
    returns: g.string(),
    resolver: 'file'
  }
})
```

### Environment variables

Node's `process.env` return nullable strings, which are a bit annoying to use in fields requiring non-nullable values. The schema has a helper `g.env()` that throws if the variable is not set, and returns a guaranteed string.

```ts
const github = connector.GraphQL('GitHub', {
  url: 'https://api.github.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', `Bearer ${g.env('GITHUB_TOKEN')}`)
  }
})
```

When working locally with Grafbase CLI you must set the environment variables inside `grafbase/.env`.
