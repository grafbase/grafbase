# **![Grafbase](https://github.com/grafbase/grafbase/assets/14347895/9580d0f7-d50f-4d30-8dd0-dcea1a83409e)** **Grafbase**

**The High-Performance GraphQL Federation Platform for Mission-Critical APIs**

[Website](https://grafbase.com) • [Documentation](https://grafbase.com/docs) • [CLI](https://grafbase.com/cli) • [Community](https://grafbase.com/community) • [Blog](https://grafbase.com/blog)

## **What is Grafbase?**

Grafbase is a self-hosted, Rust-powered GraphQL Federation Gateway designed for high-scale, mission-critical applications. Whether you're unifying microservices, legacy systems, or third-party APIs—Grafbase helps teams ship faster and more safely.

### **Built for Federation v2**

* **Federation-first architecture** — Native support for Apollo Federation v2 specification and the upcoming Composite Schemas spec  
* **40% faster performance** — Rust-powered engine delivers superior speed  
* **Extensible via WebAssembly** — Customize authentication, authorization, request lifecycle, and include arbitrary APIs and data sources into your federated graph with resolver extensions  
* **AI-native with MCP** — First GraphQL gateway with built-in Model Context Protocol server support to turn your GraphQL API into a full fledged MCP server

## **Why choose Grafbase for GraphQL Federation?**

### **Superior performance**

Grafbase delivers up to 40% faster response times vs Apollo and other gateways with lower memory usage and CPU consumption. Built in Rust for maximum efficiency.

### **Enterprise-grade security**

* Advanced schema governance with the Grafbase Dashboard at app.grafbase.com, with schema checks  
* Fine-grained authorization and authentication in the Gateway  
* Rate limiting, operation limits, and trusted documents  
* SOC 2 Type II compliant

### **Flexible deployment options for the Enterprise Platform**

* **Self-hosted**: Full control in your infrastructure  
* **Managed Cloud**: Grafbase-hosted  
* **Air-gapped**: Offline deployments for high-security environments

The Gateway itself is always 100% self hosted.

### **Universal data integration**

Connect any data source through GraphQL Federation:

* GraphQL [subgraphs](https://grafbase.com/docs/gateway/configuration/subgraph-configuration)  
* [REST APIs](https://grafbase.com/extensions/rest)  
* [gRPC services](https://grafbase.com/extensions/grpc)  
* Databases ([Postgres](https://grafbase.com/extensions/postgres), [Snowflake](https://grafbase.com/extensions/snowflake))  
* Message queues ([Kafka](https://grafbase.com/extensions/kafka), [NATS](https://grafbase.com/extensions/nats))  
* Custom protocols and data sources via [WebAssembly extensions](https://grafbase.com/docs/gateway/extensions)

**Core features**

| Apollo Federation v2 | Native support for Apollo Federation v2 spec |
| **Rust-Powered Gateway** | Ultra-low latency and memory efficiency at enterprise scale |
| **Extensions** | Customize auth, transforms, and business logic without gateway modifications |
| **Schema Governance Platform** | Composition checks, breaking change detection, and approval workflows with schema proposals (in the control plane) |
| **Branch Environments Platform** | Schema versioning and branch-aware development environments (in the control plane) |
| **CLI & Gateway** | Complete toolchain for development, deployment, and management |
| **MCP Integration** | Built-in Model Context Protocol support to efficiently expose your GraphQL API as an MCP server with a few lines of configuration |
| **Observability (in the Platform)** | Traces, metrics, logs, and operation analytics |

## Repository overview

This repository contains the core open source components of Grafbase: the CLI, the Gateway and supporting libraries. See the [grafbase/extensions](https://github.com/grafbase/extensions) repository for our open source extensions.

## **🚀 Quick start**

Get started quickly with Grafbase by following our [getting started guide](https://grafbase.com/guides/introduction-to-graphql-federation).

## **Examples & templates**

Explore real-world implementations and integration patterns in our [examples](https://github.com/grafbase/grafbase/tree/main/examples) directory:

**Learn more:**

* [gRPC Services](https://grafbase.com/changelog/grpc-extension) \- Protocol Buffer service integration  
* [WASM Extensions](https://grafbase.com/docs/features/extensions) \- Custom authentication, authorization, resolvers and request lifecycle hooks

## **Extending with WebAssembly**

Grafbase supports powerful customization via WebAssembly extensions:

```
# Create a custom authentication extension
grafbase extension init --type authentication auth-guard
cd auth-guard

# Build and install
grafbase extension build
grafbase extension install
```

**Extension Use Cases:**

* **Custom Authentication** \- JWT validation, API key management  
* **Custom Authorization**\- implement arbitrary authorization business logic, declaratively requiring data from the graph  
* **Arbitrary resolvers — plug your non-GraphQL APIs and data sources in your federated graph without writing and deploying any additional GraphQL server**   
* **Observability Hooks** \- Custom logging and metrics collection  
* **Rate Limiting** \- Advanced throttling and quota management

**Learn more:** [Extension SDK Documentation](https://grafbase.com/docs/features/extensions)

\#

## **Performance & benchmarks**

Grafbase consistently outperforms other GraphQL Federation gateways:

| Metric | Grafbase | Apollo Router |  Cosmo Router |
| :---- | :---- | :---- | :---- |
| **Response Time** | ✅ Baseline | 🟡 Slower | 🟡 Slower |
| **Memory Usage** | ✅ Efficient | 🔴 High | 🟡 moderate |
| **Cold Start** | ✅ Fast | 🟡 Slower | 🟡 Slower |
| **Throughput** | ✅ High | 🟡 Moderate | 🟡 Moderate |

See detailed benchmarks comparing Grafbase vs Apollo vs Cosmo vs Hive in our [performance analysis](https://grafbase.com/blog/benchmarking-graphql-federation-gateways).


### **Installation**

To install the Grafbase Gateway, run the following command:

```shell
curl -fsSL https://grafbase.com/downloads/gateway | bash
```

### **Deployment Modes**

**Hybrid Mode** (Connected to Grafbase Cloud): Start the gateway in hybrid mode with the graph reference and an organization access token:

```shell
GRAFBASE_ACCESS_TOKEN=token ./grafbase-gateway \
  --config grafbase.toml \
  --graph-ref graph@branch
```

**Air-gapped Mode** (Fully self-contained): Start the gateway in air-gapped mode with a local schema file:

```shell
./grafbase-gateway \
  --config /path/to/grafbase.toml \
  --schema /path/to/federated-schema.graphql \
  --listen-address 127.0.0.1:4000
```

**Docker Deployment**:

```shell
docker run -p 4000:4000 \
  -v $(pwd)/grafbase.toml:/etc/grafbase.toml \
  -v $(pwd)/schema.graphql:/etc/schema.graphql \
  ghcr.io/grafbase/gateway:latest \
  --config /etc/grafbase.toml \
  --schema /etc/schema.graphql
```

### **Gateway Features**

* JWT authentication and federated authorization  
* Rate limiting, operation limits, and trusted documents  
* Entity caching and automatic persisted queries  
* Health check endpoints and request lifecycle hooks  
* OpenTelemetry integration for logs, traces, and metrics  
* … and [many more](https://grafbase.com/docs/gateway/installation)

---

**Community & contributing**

We welcome contributions from the community\! Here's how to get involved:

### **Get help & connect**

* [Join our Discord](https://grafbase.com/community) \- Real-time community support  
* [Report Issues](https://github.com/grafbase/grafbase/issues) \- Bug reports and feature requests  
* [Documentation](https://grafbase.com/docs) \- Comprehensive guides and API reference

### **Contributing code**

1. **Fork the repository** and create a feature branch  
2. **Read our Contributing Guide** \- Development setup and guidelines  
3. **Submit a Pull Request** \- We review all contributions promptly  
4. **Join our Discord** \- Connect with maintainers and contributors

### **Ways to contribute**

* **Bug fixes** and performance improvements  
* **Documentation** improvements and examples  
* **Extensions** for new data sources and protocols  
* **Testing** and quality assurance  
* **Developer experience** enhancements

## **License**

Grafbase Gateway is licensed under the [**Mozilla Public License 2.0**](https://www.mozilla.org/en-US/MPL/2.0/) **(MPL-2.0)**.

## **Useful links**

### **Documentation & guides**

* [**Official Documentation**](https://grafbase.com/docs) \- Complete API reference and guides  
* [**GraphQL Federation Guide**](https://grafbase.com/docs/guides/introduction-to-graphql-federation) \- Learn federation concepts  
* [**Migration from Apollo**](https://grafbase.com/docs/guides/migrating-from-apollo) \- Switch from Apollo Federation

### **Tools & resources**

* [**Grafbase CLI**](https://grafbase.com/cli) \- Command-line interface and tooling  
* [**Gateway Documentation**](https://grafbase.com/docs/self-hosted-gateway) \- Self-hosted deployment guide  
* [**Benchmarks & Analysis**](https://grafbase.com/blog/benchmarking-grafbase-vs-apollo-vs-cosmo-vs-mesh) \- Performance comparisons

### **Stay connected**

* [**Website**](https://grafbase.com) \- Product information and features  
* [**Blog**](https://grafbase.com/blog) \- Technical insights and updates  
* [**Changelog**](https://grafbase.com/changelog) \- Latest features and improvements  
* [**X**](https://x.com/grafbase) \- News and community updates

