# Dynamic Schema Contracts - Healthcare Management System

This example demonstrates a federated GraphQL architecture for a healthcare management system with multiple subgraphs, two of them sharing a schema (clinics), and schema contracts, with the [tag extension](https://grafbase.com/extensions/tag) exposing different views (subsets) of the federated graph to different types of users.

## Architecture Overview

The system consists of four subgraphs:

1. **Patients Subgraph** - Manages patient demographic and insurance information
2. **Clinic A Subgraph** - Downtown Medical Center with doctors and appointments
3. **Clinic B Subgraph** - Westside Health Clinic (same schema as Clinic A, different data)
4. **Medical Records Subgraph** - Highly sensitive medical information

## Testing this

In this example project, you can act as different types of clients to the system by setting the `x-api-key` header.

- By default, you have minimal access (fields and types tagged "public").
- With an api key that starts with `patient-`, you have access to the fields available to patients.
- With an api key that starts with `doctor-`, you have access to the fields available to doctors.
- With an api key that starts with `billing-`, you have access to a restricted set of fields that is only for the billing teams.
- With an api key that starts with `admin-`, you have access to all fields.

You can pass these headers when querying, but also for introspection purposes. From the perspective of a client with a specific API key (in a real world scenario, that could be based on another header, the domain, JWT claims, etc.), only the view / slice / subset of the federated graph will exist, both for GraphQL API introspection and execution.

If you want to set a default set of included or excluded tags for the gateway, you can do so with the `graph.contracts.default_key` in the gateway configuration (`grafbase.toml`).

### Shared Schema Pattern

Clinic A and Clinic B use identical GraphQL schemas but serve different data, demonstrating how you could use schema contracts to implement multi-tenancy.

By default, the clinic data will be from clinic A. If you want to switch to clinic B, send the `x-clinic: b` header with your GraphQL requests.

## Running the Example

### Using Docker Compose

```bash
docker compose up
```

This will start:

- All four subgraphs on ports 4001-4004
- Grafbase Gateway on port 4000

### Manual Setup

1. Install dependencies for each subgraph:
```bash
cd subgraphs/patients && npm install
cd ../clinic-a && npm install
cd ../clinic-b && npm install
cd ../medical-records && npm install
```

2. Start each subgraph in separate terminals:
```bash
# Terminal 1
cd subgraphs/patients && npm start

# Terminal 2
cd subgraphs/clinic-a && npm start

# Terminal 3
cd subgraphs/clinic-b && npm start

# Terminal 4
cd subgraphs/medical-records && npm start
```

3. Start the Grafbase Gateway:
```bash
grafbase dev
```

## Testing

Run the integration tests:
```bash
hurl test.hurl
```
