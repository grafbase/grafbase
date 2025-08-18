# Query Solver Crate - Implementation Guide

## Overview

The query-solver crate is responsible for transforming GraphQL operations into optimized execution plans for federated GraphQL. It models query planning as a graph optimization problem, specifically using the Steiner Tree problem to find minimal-cost paths through a solution space.

## Core Pipeline

```
GraphQL Operation → Solution Space Generation → Steiner Tree Solving → Post-Processing → SolvedQuery
```

### 1. Solution Space Generation

- Transforms the GraphQL operation into a graph of all possible execution paths
- Creates nodes for query fields, query partitions (subgraph resolvers), and providable fields
- Establishes edges representing dependencies and requirements
- Handles complex scenarios like @provides, @requires, interfaces, and entity resolvers

### 2. Steiner Tree Solving

- Models the problem as finding the minimum-cost subgraph connecting required nodes
- Uses the GreedyFLAC algorithm (Greedy Flow Algorithm Computation) for efficient approximation
- Handles dynamic edge costs based on dispensable requirements (requirements only needed on certain paths)
- Iteratively refines the solution through cost updates and tree growth

### 3. Post-Processing

- Adjusts response keys to avoid collisions
- Ensures proper mutation execution order
- Breaks dependency cycles between query partitions
- Assigns root typename fields to appropriate resolvers

## Module Structure

### `/src/lib.rs`

Main entry point with `solve()` function that orchestrates the entire pipeline.

### `/src/query/`

Core query representation and field management:

- `Query<G, Step>`: Generic query structure parameterized by graph type and processing step
- `QueryField`: Individual field definitions with metadata
- `FieldFlags`: Bitflags for field properties (EXTRA, INDISPENSABLE, LEAF_NODE, TYPENAME, etc.)
- `Node` and `Edge` enums for graph representation

### `/src/solution_space/`

Solution space construction and management:

- **builder/**: Constructs the solution space from operation fields
  - `operation_fields.rs`: Ingests fields from the GraphQL operation
  - `providable_fields.rs`: Creates fields that can be provided by resolvers
  - `alternative.rs`: Handles multiple resolution paths
  - `prune.rs`: Removes unused resolvers
- **node.rs**: Node types (Root, QueryField, QueryPartition, ProvidableField)
- **edge.rs**: Edge types representing dependencies and requirements

### `/src/solve/`

Steiner tree solving implementation:

- **solver.rs**: Main solver coordinating the algorithm
- **steiner_tree/greedy_flac.rs**: GreedyFLAC algorithm implementation
- **input/**: Prepares input for the Steiner algorithm
- **updater.rs**: Updates edge weights based on dynamic requirements
- **solution.rs**: Converts Steiner tree solution to query representation

### `/src/post_process/`

Query optimization after solving:

- **response_key.rs**: Handles response key collision avoidance
- **mutation_order.rs**: Ensures correct mutation execution sequence
- **partition_cycles.rs**: Breaks circular dependencies between subgraphs
- **root_typename.rs**: Assigns typename fields to appropriate resolvers

## Key Algorithms

### GreedyFLAC (Greedy Flow Algorithm Computation)

The core algorithm uses a water flow analogy to find optimal paths:

1. **Flow Model**: Each terminal (required field) acts as a water source flowing at rate 1
2. **Edge Saturation**: Edges fill with flow based on their weight/cost
3. **Path Selection**: When an edge saturates, it's added to the Steiner tree
4. **Degenerate Flow Detection**: Prevents multiple paths to the same node
5. **Termination**: Stops when reaching the root (all terminals connected)

Key implementation details:

- Uses priority queue (Fibonacci heap) for efficient edge selection
- Maintains flow rates per node for quick updates
- Tracks feeding terminals to detect degenerate flows
- Time complexity: O(m log(n)k + min(m, nk)nk²)

### Dynamic Cost Updates

The solver handles dispensable requirements through iterative refinement:

1. Initial cost calibration based on current tree
2. Tree growth to include more terminals
3. Cost updates when new requirements become active
4. Fixed-point iteration until stable

## Core Data Structures

### Query Fields

```rust
pub struct QueryField {
    pub type_conditions: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub response_key: Option<ResponseKey>,
    pub definition_id: Option<FieldDefinitionId>,
    pub argument_ids: QueryOrSchemaFieldArgumentIds,
    pub location: Location,
    // ...
}
```

### Solution Space Nodes

- **Root**: Entry point for query execution
- **QueryField**: Field from the GraphQL operation
- **QueryPartition**: Subgraph resolver execution
- **ProvidableField**: Field that can be provided by a resolver

### Steiner Tree State

```rust
pub struct SteinerTree {
    pub root: NodeIndex,
    pub nodes: FixedBitSet,        // Nodes in the tree
    pub edges: FixedBitSet,        // Edges in the tree
    pub terminals: Vec<NodeIndex>,  // Required nodes
    pub total_weight: SteinerWeight // Total cost
}
```

## Important Concepts

### Indispensable vs Dispensable Requirements

- **Indispensable**: Always required (operation fields, @authorized fields)
- **Dispensable**: Only needed if certain paths are taken (@requires fields)

### Query Partitions

Represent execution segments delegated to specific subgraphs. The solver ensures:

- Minimal number of subgraph calls
- Proper data flow between partitions
- Cycle-free execution order

### Providable Fields

Fields that resolvers can provide, potentially with requirements. The system tracks:

- Which fields each resolver provides
- Requirements needed to provide those fields
- Cost of satisfying requirements

## Performance Considerations

1. **Graph Sparsity**: Solution space is kept sparse by pruning unnecessary paths early
2. **Incremental Updates**: Steiner tree grows incrementally, updating only affected parts
3. **Bitset Operations**: Extensive use of bitsets for efficient set operations
4. **Cost Caching**: Requirement costs are cached and updated only when necessary

## Testing

The crate includes comprehensive tests in `/src/tests/` covering:

- Basic query planning scenarios
- Complex federation patterns (entities, interfaces, unions)
- Edge cases (cycles, mutations, introspection)
- Performance benchmarks

## Usage Example

```rust
use query_solver::solve;
use schema::Schema;
use operation::Operation;

let schema: Schema = // ... load schema
let mut operation: Operation = // ... parse operation

let solved_query = solve(&schema, &mut operation)?;
// solved_query now contains the optimized execution plan
```

## Key Invariants

1. **Tree Property**: Solution is always a tree (no cycles)
2. **Connectivity**: All required fields are reachable from root
3. **Minimality**: No unnecessary resolvers in the solution
4. **Correctness**: All dependencies and requirements are satisfied

## Debugging

The crate provides DOT graph visualization at various stages:

- Solution space graph
- Steiner input graph
- Final solution graph

Use `to_pretty_dot_graph()` methods and visualize with Graphviz or online tools.
