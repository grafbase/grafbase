# GreedyFLAC Algorithm Summary

## Overview

GreedyFLAC is a greedy approximation algorithm for the Directed Steiner Tree (DST) problem. Given a directed weighted graph G = (V, A), a root node r, and a set of k terminal nodes X, the algorithm finds a directed tree rooted at r that spans all terminals while minimizing the total cost.

## Core Concept: Flow-Based Approach

The algorithm uses a water flow analogy:

- Each terminal acts as a water source, filling the graph through its incoming arcs
- Each arc has a volume equal to its weight/cost
- Water flows at a rate of 1 unit per second per terminal
- When an arc is completely filled (saturated), water continues flowing through the node's other incoming arcs
- The flow stops when the root is reached

## Key Definitions

- **Density**: For a tree T with cost ω(T) spanning terminals X', the density is d(T) = ω(T) / |X'|
- **Flow rate k(v)**: Number of terminals connected to node v through saturated arcs
- **Saturated arc**: An arc completely filled with flow
- **Degenerate flow**: When a terminal can reach a node through multiple distinct paths (not allowed)

## Main Algorithm Structure

### GreedyFLAC (Algorithm 2)

```
1. Initialize T = ∅
2. While terminals remain uncovered:
   a. Run FLAC to find a partial solution T₀
   b. Add T₀ to T
   c. Remove covered terminals from X
3. Return T
```

### FLAC - Flow Algorithm Computation (Algorithm 3 - Naive Version)

```
1. Initialize: GSAT = (V, ∅), t = 0, M = ∅ (marked arcs), f(a) = 0 for all arcs
2. While true:
   a. For each arc not in GSAT ∪ M: calculate time until saturation t(a) = (ω(a) - f(a))/k(a)
   b. Select arc (u,v) with minimum t(a)
   c. Update flow in all unsaturated arcs: f(a) = f(a) + k(a) × t(u,v)
   d. Update elapsed time: t = t + t(u,v)
   e. If u = r (root reached): return tree T₀ linking r to terminals
   f. If adding (u,v) creates degenerate flow: mark (u,v) and add to M
   g. Otherwise: add (u,v) to GSAT
```

## Fast Implementation (Algorithm 12)

The fast implementation improves efficiency by:

1. **Maintaining flow rates**: Store k(v) per node instead of computing from GSAT
2. **Efficient arc selection**: Use a Fibonacci heap ordered by saturation time
3. **Smart degenerate flow detection**: Use sets Xv (terminals linked to v) and Yv (partial maintenance)
4. **Sorted arc lists**: Maintain Γ⁻(v) - incoming arcs sorted by weight

### Key Data Structures

- **k(v)**: Flow rate at node v
- **Xv**: List of terminals v is connected to
- **Yv**: Set for efficient degenerate flow detection
- **t(v)**: Time when next incoming arc of v saturates
- **Γ⁻(v)**: Incoming arcs of v sorted by weight
- **ΓSAT⁻(v), ΓSAT⁺(v)**: Saturated incoming/outgoing arcs
- **F**: Fibonacci heap of nodes ordered by t(v)

### Fast FLAC Steps

```
1. Initialize data structures
2. While true:
   a. Extract node v with minimum t(v) from heap F
   b. Get first arc (u,v) from Γ⁻(v)
   c. If u = r: build and return tree T₀
   d. Check for degenerate flow using Yv ∩ Xu
   e. If not degenerate:
      - Add (u,v) to saturated arcs
      - Update k(w), Xw, Yw for all ancestors w of u
      - Update t(w) in heap for affected nodes
   f. Remove (u,v) from Γ⁻(v)
```

## Degenerate Flow Detection (Algorithm 9)

```
For each ancestor w of u in the saturated tree:
   If Yw ∩ Xv ≠ ∅:
      - Update Yw' for all nodes w' between w and u
      - Return true (flow is degenerate)
Return false
```

## Performance Characteristics

- **Approximation ratio**: k (number of terminals)
- **Time complexity**: O(m log(n)k + min(m, nk)nk²)
  - m = number of arcs
  - n = number of nodes
  - k = number of terminals
- **Space complexity**: O(nk + m)

## Key Properties

1. **Density guarantee**: The algorithm finds trees with density at most k times optimal
2. **Greedy selection**: Always selects the partial solution with best density ratio
3. **Flow conservation**: The elapsed time when root is reached equals the tree's density
4. **Non-degeneracy**: Ensures solution is always a tree (no cycles or multiple paths)

## Implementation Notes

- Use Fibonacci heap for efficient minimum extraction and key decrease operations
- Intersection checks done in O(k) using boolean arrays
- Flow rate updates propagate only to affected ancestors
- Early termination when root is reached saves unnecessary computation

# Query Solver Steiner algorithm in 'crates/engine/query-solver/src/solve/solver.rs'

The Solver implements a dynamic Steiner Tree algorithm for finding optimal query resolution paths in GraphQL. Here's how it works:

Core Problem

The solver addresses a Steiner Tree problem with dynamic edge costs based on dispensable requirements. The key challenge is that some nodes (requirements) are only needed if
certain paths are taken through the graph.

Key Components

1. Solver (solver.rs):

   - Manages the overall solving process
   - Handles dispensable requirements metadata
   - Coordinates cost updates based on path-dependent requirements
   - Orchestrates the iterative solving process

2. ShortestPathAlgorithm (shortest_path/mod.rs):

   - Implements the actual Steiner Tree construction
   - Maintains shortest paths from root to all nodes
   - Handles incremental terminal additions
   - Manages cost updates and path recalculations

3. ShortestPathSteinerTree (shortest_path/tree.rs):

   - Tracks which nodes are in the Steiner tree
   - Maintains shortest path costs and incoming edges
   - Implements tree growth operations

Algorithm Flow

1.  Initialization:

    - Identifies initial terminals (query fields marked as LEAF_NODE and INDISPENSABLE)
    - Builds a filtered graph containing only relevant nodes/edges
    - Populates dispensable requirements metadata
    - Performs initial cost calibration

2.  Iterative Execution (execute() method):
    loop {
    // Grow tree to include more terminals
    let has_terminals_left = self.algorithm.continue_steiner_tree_growth();

        // Update costs based on new requirements
        let added_new_terminals = self.cost_fixed_point_iteration()?;

        // Stop if no terminals left and no new ones added
        if !has_terminals_left && !added_new_terminals {
            break;
        }

    }

3.  Dynamic Cost Updates:

    - When a node with dispensable requirements is added to the tree, its requirements become indispensable
    - Edge costs are adjusted to reflect the cost of satisfying these requirements
    - Uses fixed-point iteration to stabilize costs

4.  Tree Growth Strategy:

    - Prioritizes zero-cost terminals first
    - Adds at least one non-zero cost terminal per iteration
    - Updates shortest paths after each growth phase
    - Handles cascading zero-cost additions

Key Interactions

- Solver → ShortestPathAlgorithm:
  - continue_steiner_tree_growth(): Advances tree construction
  - insert_edge_cost_update(): Queues edge cost changes
  - apply_all_cost_updates(): Applies pending cost changes
  - estimate_extra_cost(): Calculates requirement costs for path evaluation
  - extend_terminals(): Adds new required nodes based on chosen paths
- ShortestPathAlgorithm → ShortestPathSteinerTree:
  - grow_with_some_terminals(): Adds terminals and updates paths
  - update_shortest_paths(): Recalculates shortest paths after changes
  - node_addition_cost(): Gets cost to add a node to current tree

The algorithm elegantly handles the dynamic nature of the problem where choosing certain paths creates new requirements, which in turn affects the optimal solution. The
iterative approach with cost updates ensures that the final Steiner tree considers all path-dependent requirements while minimizing total cost.

# GreeydyFLAC query solver implementation

path: crates/engine/query-solver/src/solve/steiner_tree/greedy_flac/mod.rs

Implementation Overview

The implementation consists of two main files in /crates/engine/query-solver/src/solve/steiner_tree/greedy_flac/:

1. mod.rs: Contains the GreedyFlacAlgorithm struct that provides the high-level interface
2. flac.rs: Contains the core FLAC algorithm implementation

Key Components

GreedyFlacAlgorithm (mod.rs:19-186):

- Manages the Steiner tree construction for DAGs
- Integrates with the broader query solver context
- Provides methods for:
  - Initializing with root and terminals
  - Updating edge costs dynamically
  - Extending terminals during execution
  - Estimating costs for additional terminals
  - Generating DOT graph visualization

Flac (flac.rs:42-98):

- Core data structure maintaining:
  - Graph state (root, Steiner tree nodes, edge weights)
  - Algorithm state (saturated edges, flow rates, feeding terminals)
  - Run state (time, priority heap, stack)

Runner (flac.rs:105-322):

- Implements the actual FLAC algorithm execution
- Key methods:
  - run(): Main algorithm loop
  - update_flow_rates(): Updates flow after edge saturation
  - detect_generate_flow_and_collect_edges(): Checks for degenerate flow

Algorithm Flow

1. Initialization: Sets up the root node and initial terminals
2. Main Loop:

   - Extracts next saturating edge from priority heap
   - Updates flow rates and checks for degenerate flow
   - Adds saturated edges to the tree
   - Continues until reaching the Steiner tree (root)

3. Flow Management: Uses water flow analogy where terminals are sources flowing at rate 1

Key Features

- Efficient edge selection: Uses priority queue ordered by saturation time
- Degenerate flow detection: Prevents multiple paths to same node
- Dynamic cost updates: Supports changing edge costs during execution
- Cost estimation: Can estimate costs for hypothetical terminal additions

The implementation follows the FLAC algorithm described in the context.md, using a flow-based approach to greedily construct a Steiner tree that connects all terminals to the root
with minimal cost.
