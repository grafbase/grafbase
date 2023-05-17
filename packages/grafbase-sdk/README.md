# The Grafbase SDK

A TypeScript library to generate a Grafbase configuration. It replaces the `schema.graphql` file with `grafbase.config.ts`, which should be placed into the project's `grafbase` directory.

The configuration should define the schema, exporting the config as `default`:

```typescript
// g is a schema generator, config the final object to return
import { g, config } from '@grafbase/sdk'

// types are generated with the `type` method,
// followed by the name and fields.
const profile = g.type("Profile", {
  address: g.string(),
})

// models can be generated with the `model` method
const user = g.model("User", {
  name: g.string(),
  age: g.int().optional(),
  profile: g.ref(profile).optional(),
  parent: g.relation(() => user).optional()
})

// finally we export the default config
export default config({
  schema: g
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

## Types

Types are generated with the `type` method:

```typescript
g.type("Profile", {
  address: g.string()
})
```

## Models

Types are generated with the `model` method:

```typescript
g.model("User", {
  name: g.string()
})
```

## Enums

Enums can be generated either from TypeScript enums or from a dynamic array:

```typescript
enum Fruits {
  Apples,
  Oranges
}

g.enum('Fruits', Fruits)
```

or

```typescript
g.enum('Fruits', ['Apples', 'Oranges'])
```

An enum can be used as a field type with the `ref` method:

```typescript
const e = g.enum('Fruits', ['Apples', 'Oranges'])

g.type("User", {
  favoriteFruit: g.ref(e)
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

const input = g.type('CheckoutSessionInput', { name: g.string() })
const output = g.type('CheckoutSessionOutput', { successful: g.boolean() })

g.mutation('checkout', {
  args: { input: g.ref(input) },
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

- String: `g.string()`
- ID: `g.id()`
- Email: `g.email()`
- Int: `g.int()`
- Float: `g.float()`
- Boolean: `g.boolean()`
- Date: `g.date()`
- DateTime: `g.datetime()`
- IPAddress: `g.ipAddress()`
- Timestamp: `g.timestamp()`
- URL: `g.url()`
- JSON: `g.json()`

## Enum fields

```typescript
// first greate an enum
enum Fruits {
  Apples,
  Oranges
}

const fruits = g.enumType("Fruits", Fruits)

// then use it e.g. in a model
g.model("User", {
  favoriteFruit: g.enum(fruits)
})
```

## Reference fields

Referencing a type is with the `ref` method:

```typescript
const profile = g.type("Profile", {
  address: g.string()
})

g.model("User", {
  profile: g.ref(profile)
})
```

## Relation fields

Creating a relation between models is with the `relation` method.

```typescript
const user = g.model("User", {
  posts: g.relation(() => post).name("relationName").list()
})

const post = g.model("Post", {
  author: g.relation(user).name("relationName")
})
```

## Optional fields

By default the generated fields are _required_. To make them optional is with the `optional` method:

```typescript
const user = g.model("User", {
  posts: g.string().optional()
})
```

## List fields

List fields can be done with the `list` method:

```typescript
const user = g.model("User", {
  names: g.string().list()
})
```

By default, the list or list items are _required_. To make the items nullable, call the `optional` method to the base type:

```typescript
const user = g.model("User", {
  names: g.string().optional().list()
})
```

To make the list itself optional, call the `optional` method to the list type:

```typescript
const user = g.model("User", {
  names: g.string().list().optional()
})
```

## Unique

Unique field can be defined to certain types of fields with the `unique` method:

```typescript
const user = g.model("User", {
  name: g.string().unique()
})
```

Additional unique scope can be given as a parameter:

```typescript
const user = g.model("User", {
  name: g.string().unique(["email"]),
  email: g.string()
})
```

## Length limit

Certain field types can have a limited length:

```typescript
const user = g.model("User", {
  name: g.string().length({ min: 1, max: 255 })
})
```

## Defaults

Default values for certain field types can be given with the `default` method. The default parameters are type-checked to fit the field type:

```typescript
const user = g.model("User", {
  name: g.string().default("meow"),
  age: g.int().default(11)
})
```

## Search

Certain types of fields can be searchable:

```typescript
const user = g.model("User", {
  name: g.string().search(),
})
```

Additionally, the whole model can be searchable:

```typescript
const user = g.model("User", {
  name: g.string(),
  age: g.int()
}).search()
```

## Connectors

Connectors are created through the connector interface:

```typescript
import { connector } from '../../src/index'
```

### OpenAPI

The OpenAPI connector can be created with the `OpenAPI` method:

```typescript
const openai = connector.OpenAPI({
  schema: 'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml'
})

const stripe = connector
  .OpenAPI({
    schema: 'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
    headers: (headers) => {
      // used in client and introspection requests
      headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
      // used only in introspection requests
      headers.introspection('foo', 'bar')
    }
  })
```

Introspecting the connector namespace to the schema happens with the `datasource` method of the schema:

```typescript
g.datasource(stripe, { namespace: 'Stripe' })
g.datasource(openai, { namespace: 'OpenAI' })
```

### GraphQL

The GraphQL connector can be created with the `GraphQL` method:

```typescript
const contentful = connector.GraphQL({
  url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}',
  headers: (headers) => {
    headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
    headers.static('Method', 'POST')
  }
})

const github = connector.GraphQL({
  url: 'https://api.github.com/graphql'
})
```

Introspecting the connector namespace to the schema happens with the `introspect` method of the schema:

```typescript
g.datasource(contentful, { namespace: 'Contentful' })
g.datasource(github, { namespace: 'GitHub' })
```
