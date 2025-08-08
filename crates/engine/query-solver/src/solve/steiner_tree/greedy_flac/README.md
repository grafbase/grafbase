# GreedyFLAC

Watel, D., & Weisser, M. A. (2016). A practical greedy approximation for the directed steiner tree problem. Journal of Combinatorial Optimization, 32(4), 1327-1370.
https://www.researchgate.net/profile/Dimitri-Watel/publication/307916063_A_practical_greedy_approximation_for_the_directed_Steiner_tree_problem/links/5f04a382299bf18816082829/A-practical-greedy-approximation-for-the-directed-Steiner-tree-problem.pdf

## Overview

The Query Solver uses the GreedyFLAC algorithm to find a good query resolution paths in GraphQL federation. This algorithm solves the Directed Steiner Tree problem - finding the minimum-cost tree that connects a root node to all required terminal nodes in a directed graph.

## The Water Flow Analogy

FLAC (FLow Algorithm Computation) uses an intuitive water flow metaphor to build the Steiner tree:

1. **Terminals as Water Sources**: Each terminal node acts as a water source, continuously pouring water at 1 unit/second
2. **Edges as Pipes**: Each edge has a capacity equal to its cost/weight
3. **Saturation**: When an edge is completely filled with water, it becomes "saturated" and part of the solution
4. **Flow Propagation**: Water flows backward through the graph (from terminals toward the root) until reaching the root

The GreedyFLAC simply applies the FLAC algorithm as many times as necessary to build a steiner tree with all terminals.

## Simple Example

Consider finding the cheapest way to connect a root server to three data terminals (T1, T2, T3):

```
        Root
       /    \
    $5/      \$8
     /        \
    A          B
   / \          \
$2/   \$3        \$7
 /     \          \
T1      T2        T3
```

### How GreedyFLAC Solves This:

**Step 1: Initialize Flow**

- T1, T2, and T3 each start flowing water at 1 unit/second
- T1 flows into edge (A→T1) with cost $2
- T2 flows into edge (A→T2) with cost $3
- T3 flows into edge (B→T3) with cost $7

```
        Root
       /    \
    $5/      \$8
     /        \
    A          B
   / \          \
$2/   \$3        \$7
 ↑     ↑          ↑
T1     T2        T3
(1/s)  (1/s)     (1/s)
```

**Step 2: First Saturation (t=2s)**

- Edge (A→T1) saturates after 2 seconds (2 units filled)
- A now has flow rate of 1 unit/second from T1
- Water from T1 continues through A's incoming edge (Root→A)

```
        Root
       /    \
    $5/      \$8
     ↑        \
    A(1/s)     B
   /=\          \
$2/   \$3        \$7
 /     ↑          ↑
T1     T2        T3
       (1/s)     (1/s)

Legend: = saturated edge
```

**Step 3: Second Saturation (t=3s)**

- Edge (A→T2) saturates after 3 seconds
- A now has flow rate of 2 units/second (from T1 and T2)
- Water flows faster through (Root→A)

```
        Root
       /    \
    $5/      \$8
     ↑        \
    A(2/s)     B
   /=\=         \
$2/   \$3        \$7
 /     \          ↑
T1     T2        T3
                 (1/s)
```

**Step 4: Root Reached (t=4.5s)**

- Edge (Root→A) saturates after 4.5 seconds total
  - First 2 seconds: 1 unit/sec × 2 sec = 2 units
  - Next 1.5 seconds: 2 units/sec × 1.5 sec = 3 units
  - Total: 5 units (edge capacity)
- Root reached! Partial tree for T1 and T2 complete

```
        Root
       /====\
    $5/      \$8
     /        \
    A          B
   /=\=         \
$2/   \$3        \$7
 /     \          ↑
T1     T2        T3
                 (1/s)
```

**Step 5: Continue for T3 (t=7s)**

- Edge (B→T3) saturates after 7 seconds
- B starts flowing at 1 unit/second into (Root→B)

```
        Root
       /====\
    $5/      \$8
     /        ↑
    A        B(1/s)
   /=\=        \=
$2/   \$3       \$7
 /     \         \
T1     T2        T3
```

**Step 6: Final Saturation (t=15s)**

- Edge (Root→B) saturates after 8 more seconds (8 units to fill)
- All terminals are now connected!

```
        Root
       /====\====
    $5/      \$8
     /        \
    A          B
   /=\=         \=
$2/   \$3        \$7
 /     \          \
T1     T2         T3
```

**Final Tree Cost**: $5 + $2 + $3 + $8 + $7 = **$25**

## Degenerate Flow Prevention

A critical aspect of GreedyFLAC is preventing "degenerate flow" - situations where water from the same terminal reaches a node through multiple paths. This would create cycles in our tree, which we must avoid.

### Example of Degenerate Flow:

```
     Root
       |
      $10
       |
       A
      / \
   $2/   \$3
    /     \
   B       C
    \     /
   $1\   /$1
      \ /
       T1
```

Without degenerate flow detection:

1. T1 would flow into both (B→T1) and (C→T1)
2. Both edges would saturate after 1 second
3. Both B and C would start flowing toward A
4. When A is reached, it would receive flow from the same terminal (T1) via two different paths
5. This creates a cycle, not a tree!

**Solution**: When the algorithm detects that adding an edge would create degenerate flow (same terminal reaching a node through multiple paths), it marks that edge as forbidden and continues with alternative paths. This ensures the result is always a valid tree structure.

In the example above, the algorithm would:

1. Allow one path (e.g., A→B→T1) to saturate normally
2. Detect that allowing A→C→T1 would create degenerate flow
3. Mark edge (C→T1) as forbidden
4. Find an alternative solution that maintains the tree property

This guarantees that the final Steiner tree has no cycles and each terminal is connected to the root through exactly one path.
