# NATS Extension

This is a NATS extension for the Grafbase Gateway. It allows you to define NATS endpoints and map them to GraphQL fields.

This serves as an example of how to build extensions dealing with pub/sub services, but it also functions as a fully operational extension you can use right now, or use as a starting point for your own extensions.

This extension expects JSON payloads. If you use a different format, fork the extension and modify it to fit your needs. For static formats such as Protobuf, we recommend customizing the extension.

## Installing

Add the following to your gateway configuration ("grafbase.toml"):

```toml
[extensions.nats]
version = "0.2"
```

Then run `grafbase extension install`. The extension will be installed in the `grafbase_extensions` directory. That directory must be present when the gateway is started.

## Building from source

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
[extensions.nats]
path = "/path/to/build"
```

## Configuration

Configure the extension through the Grafbase Gateway configuration file:

```toml
[extensions.nats]
path = "/path/to/build"

[[extensions.nats.config.endpoint]]
name = "default"
servers = ["nats://localhost:4222"]
```

- `name`: The name of the endpoint. This identifies the endpoint in the GraphQL schema. Default is `default`. You can omit the name in the configuration and in the schema if using only one endpoint.
- `servers`: The list of NATS servers to connect to.

The authentication configuration is optional, and we support multiple authentication methods:

### Password Authentication

```toml
[extensions.nats.config.endpoint.authentication]
username = "grafbase"
password = "grafbase"
```

- `username`: The username to use for authentication.
- `password`: The password to use for authentication.

[NATS documentation](https://docs.nats.io/using-nats/developer/connecting/userpass)

### Token Authentication

```toml
[extensions.nats.config.endpoint.authentication]
token = "TOKEN"
```

- `token`: The token to use for authentication.

[NATS documentation](https://docs.nats.io/using-nats/developer/connecting/token)

### Credentials Authentication

```toml
[extensions.nats.config.endpoint.authentication]
credentials = "contents of credentials file"
```

- `credentials`: The contents of the credentials file to use for authentication.

[NATS documentation](https://docs.nats.io/using-nats/developer/connecting/creds)

## Publish

To publish messages to a NATS topic, use the `@natsPublish` directive:

```graphql
directive @natsPublish(
  provider: String! = "default"
  subject: UrlTemplate!
  body: Body! = { selection: "*" }
) on FIELD_DEFINITION
```

- `provider`: The NATS provider to use. This identifies the provider in the GraphQL schema. Default is `default`. You can omit the provider in the configuration and in the schema if using only one provider.
- `subject`: The subject to publish to. This supports templating using GraphQL arguments: `{{args.myArgument}}`.
- `body`: The body of the message to publish. If not set, takes the body from the field's `input` argument. Can also be set to a static JSON object.

### Example

```graphql
type Mutation {
  publishUserEvent(id: Int!, input: UserEventInput!): Boolean! @natsPublish(
    subject: "publish.user.{{args.id}}.events"
  )
}

input UserEventInput {
  email: String!
  name: String!
}
```

This example publishes an event to a subject named `publish.user.<id>.events`. The `id` comes from the value provided in the `id` argument. The payload comes from the `input` argument:

```graphql
mutation PublishUserEvent($id: Int!, $email: String!, $name: String!) {
  publishUserEvent(id: $id, input: { email: $email, name: $name })
}
```

By calling the mutation with id `1`, email `john@example.com`, and name `John Doe`, the following message will publish to the subject `publish.user.1.events`:

```json
{
  "email": "john@example.com",
  "name": "John Doe"
}
```

## Subscribe

To subscribe to messages from a NATS topic, use the `@natsSubscription` directive:

```graphql
directive @natsSubscription(
  provider: String! = "default"
  subject: UrlTemplate!
  selection: String
  streamConfig: NatsStreamConfiguration
) on FIELD_DEFINITION
```

- `provider`: The NATS provider to use. Default is `default`.
- `subject`: The subject to subscribe to. This supports templating using GraphQL arguments: `{{args.myArgument}}`.
- `selection`: Selection to apply to the subscription payload. In [jq syntax](https://jqlang.org/manual/). This supports templating using GraphQL arguments: `{{args.myArgument}}`.
- `streamConfig`: Stream configuration for JetStream subscriptions.

If you define the `streamConfig` settings, the subscription will create a [JetStream subscription](https://docs.nats.io/nats-concepts/jetstream):

```graphql
input NatsStreamConfiguration {
  streamName: String!
  consumerName: String!
  durableName: String
  description: String
  deliverPolicy: NatsStreamDeliverPolicy! = { type: ALL }
  inactiveThresholdMs: Int! = 30000
}
```

- `streamName`: The stream name for the subscription, defines which stream to pull messages from.
- `consumerName`: The consumer name for the subscription.
- `durableName`: Setting this will cause the consumer to be "durable". The JetStream server remembers the consumer's progress for fault tolerance. If a consumer crashes, it can resume processing where it left off.
- `description`: Description of the consumer.
- `deliverPolicy`: Delivery policy for the subscription. Default is `{ type: ALL }`.
- `inactiveThresholdMs`: Threshold in milliseconds after which a consumer is considered inactive. Default is `30000`.

The delivery policy configuration for NATS streams:

```graphql
input NatsStreamDeliverPolicy {
  type: NatsStreamDeliverPolicyType!
  startSequence: Int
  startTimeMs: Int
}
```

- `type`: The type of delivery policy.
- `startSequence`: Starting sequence number for `BY_START_SEQUENCE` policy.
- `startTimeMs`: Starting time in milliseconds for `BY_START_TIME` policy.

The delivery policy types:

```graphql
enum NatsStreamDeliverPolicyType {
  ALL
  LAST
  NEW
  BY_START_SEQUENCE
  BY_START_TIME
  LAST_PER_SUBJECT
}
```

- `ALL`: Causes the consumer to receive the oldest messages still present in the system. This is the default.
- `LAST`: Will start the consumer with the last sequence received.
- `NEW`: Will only deliver new messages that the JetStream server receives after creating the consumer.
- `BY_START_SEQUENCE`: Will look for a defined starting sequence using the consumer's configured `startSequence` parameter.
- `BY_START_TIME`: Will select the first message with a timestamp after the consumer's configured `startTimeMs` parameter.
- `LAST_PER_SUBJECT`: Will start the consumer with the last message for all subjects received.

### Example

#### Basic Subscription

```graphql
type Subscription {
  userEvents(userId: Int!): UserEvent! @natsSubscription(
    subject: "user.{{args.userId}}.events"
  )
}

type UserEvent {
  type: String!
  userId: Int!
  timestamp: String!
  data: JSON
}
```

This example subscribes to a subject named `user.<userId>.events`. The `userId` comes from the value provided in the `userId` argument. When someone publishes a message to this subject, clients that have subscribed using this GraphQL subscription will receive it.

#### JetStream Subscription

```graphql
type Subscription {
  orderUpdates: OrderUpdate! @natsSubscription(
    subject: "orders.>",
    streamConfig: {
      streamName: "ORDERS",
      consumerName: "order-processor",
      durableName: "order-updates",
      deliverPolicy: { type: LAST }
    }
  )
}

type OrderUpdate {
  orderId: String!
  status: String!
  updatedAt: String!
}
```

This example creates a JetStream subscription for the `orders.>` wildcard subject, using the `ORDERS` stream. It configures a durable consumer named `order-updates` with the policy to deliver the last message received. This works well for scenarios where you only care about the latest state of each order.

#### Using Selection

```graphql
type Subscription {
  highValueTransactions: Transaction! @natsSubscription(
    subject: "banking.transactions",
    selection: "select(.amount > 1000)"
  )
}

type Transaction {
  id: String!
  amount: Float!
  accountId: String!
  timestamp: String!
}
```

This example subscribes to the `banking.transactions` subject but filters the incoming messages using a JQ-style selection to only deliver transactions with an amount greater than 1000. This enables server-side filtering of messages before sending them to the client.

The selection also supports dynamic parameters:

```graphql
type Subscription {
  transactionsAboveThreshold(minimumAmount: Float!): Transaction! @natsSubscription(
    subject: "banking.transactions",
    selection: "select(.amount > {{args.minimumAmount}})"
  )
}

type Transaction {
  id: String!
  amount: Float!
  accountId: String!
  timestamp: String!
}
```

This example subscribes to the `banking.transactions` subject and filters the incoming messages using a dynamic threshold. The [jq-style](https://jqlang.org/manual/) selection uses the `minimumAmount` argument provided by the client to only deliver transactions with an amount greater than the specified threshold. This allows clients to set their own filtering criteria when subscribing to transaction events.

## Request/Reply

A request/reply example demonstrates how to use the `@natsRequest` directive to send a request message and receive a response message from a consumer.

```graphql
directive @natsRequest(
  provider: String! = "default"
  subject: UrlTemplate!
  selection: UrlTemplate
  body: Body! = { selection: "*" }
  timeoutMs: Int! = 5000
) on FIELD_DEFINITION
```

- `provider`: The NATS provider to use. Default is `default`.
- `subject`: The subject to publish to. This supports templating using GraphQL arguments: `{{args.myArgument}}`.
- `selection`: Selection to apply to the subscription payload. In [jq syntax](https://jqlang.org/manual/). This supports templating using GraphQL arguments: `{{args.myArgument}}`.
- `body`: The body of the message to publish.
- `timeoutMs`: Timeout in milliseconds for the request. If the request does not receive a response within this time, the request will fail with a timeout error. Default is `5000`.

### Example

```graphql
type Query {
  getUserDetails(id: String!): UserDetails! @natsRequest(
    subject: "user.details.{{args.id}}",
    timeoutMs: 2000
  )
}

type UserDetails {
  id: String!
  name: String!
  email: String!
  createdAt: String!
  role: String!
}
```

This example sends a request to the subject `user.details.<id>` and waits for a response. The query uses the `id` parameter to construct the subject. The request will timeout after 2 seconds if no response arrives.

You could also send a payload with the request:

```graphql
type Query {
  authenticateUser(input: AuthInput!): AuthResponse! @natsRequest(
    subject: "auth.service",
    timeoutMs: 3000
  )
}

input AuthInput {
  username: String!
  password: String!
}

type AuthResponse {
  token: String
  userId: String
  success: Boolean!
  message: String
}
```

In this example, the authentication credentials go to the `auth.service` subject, and the service responds with authentication details. The request will timeout after 3 seconds if no response arrives.

## Key-value Store

A key-value example demonstrates how to use the `@natsKeyValue` directive to store and retrieve data from NATS JetStream key-value storage.

```graphql
directive @natsKeyValue(
  provider: String! = "default"
  bucket: UrlTemplate!
  key: UrlTemplate!
  action: NatsKeyValueAction!
  body: Body = { selection: "*" }
  selection: UrlTemplate
) on FIELD_DEFINITION
```

- `provider`: The NATS provider to use
- `bucket`: The bucket name to operate on. This supports templating using GraphQL arguments: `{{args.myArgument}}`
- `key`: The key name to operate on. This supports templating using GraphQL arguments: `{{args.myArgument}}`
- `action`: The key-value operation to perform
- `body`: The body of the message to put or create (only used for PUT and CREATE actions)
- `selection`: Selection to apply to the response payload. In jq syntax. (only used for GET action)

Supported actions:

```graphql
enum NatsKeyValueAction {
  CREATE
  PUT
  GET
  DELETE
}
```

- `CREATE`: Create a new key-value pair. Fails if the key already exists.
- `PUT`: Put a value for the key, creating it if it doesn't exist or updating it if it does.
- `GET`: Get the value for the specified key.
- `DELETE`: Delete the specified key-value pair.

The field returns a boolean value, indicating whether the operation succeeded.

### Example

This example stores a user profile in the "user-profiles" bucket using the userId as the key. If the value exists, it gets updated. The profile data comes from the input argument. The return value is the sequence number of the operation.

```graphql
type Mutation {
  saveUserProfile(userId: String!, profile: UserProfileInput!): String! @natsKeyValue(
    bucket: "user-profiles",
    key: "{{args.userId}}",
    action: PUT
  )
}

input UserProfileInput {
  name: String!
  email: String!
  preferences: JSON
  lastUpdated: String!
}
```

This example stores a user profile in the "user-profiles" bucket using the userId as the key. If the value exists, the mutation returns an error. The profile data comes from the input argument. The return value is the sequence number of the operation.

```graphql
type Mutation {
  saveUserProfile(userId: String!, profile: UserProfileInput!): String! @natsKeyValue(
    bucket: "user-profiles",
    key: "{{args.userId}}",
    action: CREATE
  )
}

input UserProfileInput {
  name: String!
  email: String!
  preferences: JSON
  lastUpdated: String!
}
```

This example retrieves a user profile from the "user-profiles" bucket using the userId as the key.

```graphql
type Query {
  getUserProfile(userId: String!): UserProfile @natsKeyValue(
    bucket: "user-profiles",
    key: "{{args.userId}}",
    action: GET
  )
}

type UserProfile {
  name: String!
  email: String!
  preferences: JSON
  lastUpdated: String!
}
```

This example deletes a user profile from the "user-profiles" bucket using the userId as the key. The return value is true if the operation succeeded.

```graphql
type Mutation {
  deleteUserProfile(userId: String!): Boolean! @natsKeyValue(
    bucket: "user-profiles",
    key: "{{args.userId}}",
    action: DELETE
  )
}
```

This example retrieves only the preferences field from a user profile using [jq-style](https://jqlang.org/manual/) selection.

```graphql
type Query {
  getUserPreferences(userId: String!): JSON @natsKeyValue(
    bucket: "user-profiles",
    key: "{{args.userId}}",
    action: GET,
    selection: ".preferences"
  )
}
```
