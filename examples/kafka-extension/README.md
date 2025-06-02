# Kafka Extension Example

This example demonstrates how to use the Kafka extension with Grafbase Gateway to integrate Apache Kafka messaging into your GraphQL API. The extension allows you to publish messages to Kafka topics and subscribe to real-time message streams using GraphQL.

## Overview

The Kafka extension enables you to:
- **Publish messages** to Kafka topics using GraphQL mutations
- **Subscribe to messages** from Kafka topics using GraphQL subscriptions
- **Support multiple authentication methods** including SASL/PLAIN and SASL/SCRAM
- **Filter messages** by key patterns
- **Configure producers and consumers** with advanced settings

## Prerequisites

- Docker and Docker Compose
- Either [Grafbase CLI](https://grafbase.com/docs/cli/installation) installed
- Basic understanding of GraphQL and Apache Kafka

## Quick Start

1. **Clone the repository and navigate to the example:**
   ```bash
   cd grafbase/examples/kafka-extension
   ```

2. **Start Kafka using Docker Compose:**
   ```bash
   docker compose up -d
   ```

   This starts:
   - Kafka broker with KRaft mode (no Zookeeper required)
   - Support for multiple authentication methods on different ports
   - Automatic topic creation and SCRAM user setup

3. **Either start the CLI in dev mode:**
   ```bash
   grafbase dev
   ```

   The gateway will be available at `http://localhost:5000/graphql`, and you can test the GraphQL mutations and subscriptions in `http://localhost:5000/`.

## Configuration

### Kafka Extension Configuration (grafbase.toml)

```toml
[extensions.kafka]
version = "0.1.1"

[[extensions.kafka.config.endpoint]]
bootstrap_servers = ["localhost:9094"]

[extensions.kafka.config.endpoint.authentication]
type = "sasl_scram"
username = "testuser"
password = "testuser-secret"
mechanism = "sha512"

[subgraphs.kafka]
schema_path = "subgraph.graphql"
```

The example is configured to use SASL/SCRAM authentication with SHA-512. The Kafka broker exposes multiple ports:
- **9092**: PLAINTEXT (no authentication)
- **9093**: SASL/PLAIN authentication
- **9094**: SASL/SCRAM authentication (used in this example)

### GraphQL Schema

The example schema demonstrates three key directives:

1. **@kafkaProducer** - Defines a Kafka producer at the schema level:
   ```graphql
   @kafkaProducer(name: "test-producer", topic: "test-topic")
   ```

2. **@kafkaPublish** - Publishes messages through the configured producer:
   ```graphql
   publishUserEvent(id: Int!, input: UserEventInput!): Boolean!
     @kafkaPublish(producer: "test-producer", key: "publish.user.{{args.id}}")
   ```

3. **@kafkaSubscription** - Subscribes to Kafka topics:
   ```graphql
   allUserEvents: UserEvent
     @kafkaSubscription(topic: "test-topic", consumerConfig: { maxWaitTimeMs: 10000 })
   ```

## Running the Example

### 1. Test Basic Connectivity

First, verify that Kafka is running and accessible:

```bash
# Check if Kafka container is healthy
docker compose ps
```

### 2. Publish a Message

Use the GraphQL mutation to publish a user event:

```graphql
curl -X POST http://localhost:5000/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { publishUserEvent(id: 1, input: { email: \"test@example.com\", name: \"Test User\" }) }"
  }'
```

The message will be published to the `test-topic` with the key `publish.user.123`.

### 3. Subscribe to Messages

Subscribe to all user events:

```bash
curl -N http://localhost:5000/graphql \
  --header 'Accept: text/event-stream' \
  --header "Content-Type: application/json" \
  --data '{"query": "subscription { allUserEvents { email name } }", "variables": {}}'
```

Or subscribe to events for a specific user ID (note: this uses the topic name as configured):

```bash
curl -N http://localhost:5000/graphql \
  --header 'Accept: text/event-stream' \
  --header "Content-Type: application/json" \
  --data '{"query": "subscription { filteredUserEvents(id: 123) { email name } }", "variables": {}}'
```

## Authentication Details

The Docker setup creates the following users for testing:

- **Admin user**: `admin` / `admin-secret` (has full permissions)
- **Test user**: `testuser` / `testuser-secret` (used in the example configuration)

These users are created with SCRAM-SHA-512 authentication mechanism.

## Advanced Configuration

### Producer Configuration

You can customize the producer behavior by modifying the `@kafkaProducer` directive:

```graphql
@kafkaProducer(
  name: "my-producer"
  topic: "my-topic"
  config: {
    compression: GZIP
    partitions: [0, 1, 2]
    batch: {
      lingerMs: 5
      maxSizeBytes: 8192
    }
  }
)
```

### Consumer Configuration

Customize consumer behavior in subscriptions:

```graphql
@kafkaSubscription(
  topic: "my-topic"
  consumerConfig: {
    startOffset: { preset: EARLIEST }
    maxBatchSize: 100
    maxWaitTimeMs: 5000
    partitions: [0, 1, 2]
  }
)
```

## Troubleshooting

### Common Issues

1. **Connection refused errors:**
   - Ensure Docker containers are running: `docker compose ps`
   - Check if the correct port is being used (9094 for SCRAM)
   - Verify the Kafka broker is healthy: `docker compose logs kafka`

2. **Authentication failures:**
   - Verify the username and password in `grafbase.toml`
   - Check that SCRAM users were created: `docker compose logs kafka-scram-users`

3. **No messages received in subscriptions:**
   - Ensure messages are being published to the correct topic
   - Check consumer configuration, especially `startOffset`
   - Verify the key filter pattern matches your message keys

### Debugging

Enable debug logging in the gateway:

```bash
env RUST_LOG=debug grafbase dev
```

Monitor Kafka logs:

```bash
docker compose logs -f kafka
```

## Cleanup

To stop and remove all containers and volumes:

```bash
docker compose down -v
```

## Further Reading

- [Kafka Extension Documentation](https://grafbase.com/extensions/kafka)
- [Grafbase Gateway Documentation](https://grafbase.com/docs/gateway)
- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)
