# Delete Subgraph Mutation Usage

This document demonstrates how to use the delete subgraph mutation in the Grafbase CLI.

## CLI Command

To delete a subgraph, use the following command:

```bash
grafbase subgraph delete <account>/<graph>@<branch> <subgraph-name>
```

### Examples

Delete a subgraph named "users" from the main branch:
```bash
grafbase subgraph delete my-account/my-graph@main users
```

Delete a subgraph from a specific branch:
```bash
grafbase subgraph delete my-account/my-graph@feature-branch products
```

## GraphQL Mutation

The underlying GraphQL mutation that gets executed:

```graphql
mutation DeleteSubgraph($input: DeleteSubgraphInput!) {
  deleteSubgraph(input: $input) {
    ... on DeleteSubgraphSuccess {
      __typename
    }
    ... on SubgraphNotFoundError {
      __typename
    }
    ... on GraphDoesNotExistError {
      __typename
    }
    ... on GraphNotFederatedError {
      __typename
    }
    ... on GraphBranchDoesNotExistError {
      __typename
    }
    ... on FederatedGraphCompositionError {
      messages
    }
    ... on DeleteSubgraphDeploymentFailure {
      deploymentError
    }
  }
}
```

### Variables

```json
{
  "input": {
    "accountSlug": "my-account",
    "graphSlug": "my-graph",
    "branch": "main",
    "subgraph": "users",
    "dryRun": false
  }
}
```

## Error Handling

The mutation returns different error types:

- **SubgraphNotFoundError**: The specified subgraph doesn't exist
- **GraphDoesNotExistError**: The specified graph doesn't exist
- **GraphNotFederatedError**: The graph is not a federated graph
- **GraphBranchDoesNotExistError**: The specified branch doesn't exist
- **FederatedGraphCompositionError**: The deletion would break federation composition
- **DeleteSubgraphDeploymentFailure**: The deployment failed after deletion

## Implementation Details

The delete subgraph functionality is implemented in:

1. **CLI Command Handler**: `grafbase/cli/src/subgraph.rs`
   - Handles the CLI command and calls the API

2. **API Client**: `grafbase/cli/src/api/subgraph.rs`
   - Makes the GraphQL request to delete the subgraph

3. **GraphQL Types**: `grafbase/cli/src/api/graphql/types/mutations/delete_subgraph.rs`
   - Defines the mutation structure and response types

4. **CLI Input**: `grafbase/cli/src/cli_input/subgraph.rs`
   - Defines the command structure with subcommands for list and delete