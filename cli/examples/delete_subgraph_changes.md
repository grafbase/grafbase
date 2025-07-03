# Delete Subgraph Mutation Implementation Summary

This document summarizes the changes made to implement the delete subgraph mutation in the Grafbase CLI.

## Files Created

### 1. `grafbase/cli/src/api/graphql/types/mutations/delete_subgraph.rs`
New file containing the GraphQL mutation types for deleting a subgraph:
- `DeleteSubgraphArguments` - Variables for the mutation
- `DeleteSubgraphInput` - Input type matching the GraphQL schema
- `DeleteSubgraphMutation` - The mutation query fragment
- `DeleteSubgraphPayload` - Union type for all possible responses
- Various error types (reusing some from parent module)

## Files Modified

### 2. `grafbase/cli/src/api/graphql/types/mutations.rs`
- Added `pub(crate) mod delete_subgraph;` to export the new module

### 3. `grafbase/cli/src/api/subgraph.rs`
- Updated imports to include `MutationBuilder` and the new delete subgraph types
- Modified the `delete` function (previously `delete_subgraph`) to:
  - Use the proper mutation structure with `DeleteSubgraphInput`
  - Handle all possible response variants from the union type
  - Provide appropriate error messages for each error case

### 4. `grafbase/cli/src/cli_input/subgraph.rs`
- Completely restructured to support subcommands:
  - Changed from a simple struct to include a `SubgraphSubCommand` enum
  - Added `List` and `Delete` subcommands
  - Each subcommand has its own arguments (graph_ref, and name for delete)

### 5. `grafbase/cli/src/cli_input.rs`
- Updated exports to include `SubgraphSubCommand`

### 6. `grafbase/cli/src/main.rs`
- Modified the `SubCommand::Subgraph` match arm to handle both list and delete subcommands
- Routes to appropriate functions based on the subcommand

### 7. `grafbase/cli/src/subgraph.rs`
- Updated both `list` and `delete` functions to work with the new command structure
- Added proper error handling for missing branch parameter in delete

### 8. `grafbase/cli/src/output/report.rs`
- Added `subgraph_delete_success` function to report successful deletion

## Key Design Decisions

1. **Union Type Handling**: The delete mutation returns a union type with multiple possible responses. Each response type is handled explicitly with appropriate error messages.

2. **Subcommand Structure**: Instead of having separate top-level commands, we use subcommands under `grafbase subgraph` for better organization:
   - `grafbase subgraph list <graph-ref>`
   - `grafbase subgraph delete <graph-ref> <name>`

3. **Branch Requirement**: For delete operations, the branch is required (not optional) to ensure users explicitly specify which branch they're deleting from.

4. **Error Types**: Reused existing error types where possible (e.g., `GraphDoesNotExistError`, `FederatedGraphCompositionError`) to maintain consistency.

5. **Dry Run Support**: The mutation supports a `dry_run` parameter (set to `false` by default) which could be exposed as a CLI flag in the future.

## Usage

```bash
# List subgraphs
grafbase subgraph list account/graph@branch

# Delete a subgraph
grafbase subgraph delete account/graph@branch subgraph-name
```

## Testing Recommendations

1. Test successful deletion of an existing subgraph
2. Test error handling for non-existent subgraphs
3. Test error handling for non-federated graphs
4. Test composition errors when deletion would break the schema
5. Test with missing branch parameter
6. Test with non-existent graph or branch