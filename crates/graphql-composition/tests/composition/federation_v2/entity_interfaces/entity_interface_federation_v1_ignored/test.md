Federation v1 subgraphs should not be considered for entity interface detection.

This test verifies that when:
- Two federation v2 subgraphs define a regular interface `Item` without @key 
- One federation v1 subgraph defines the same interface with @key

The interface remains a regular interface and is NOT promoted to an entity interface, because:
- Entity interfaces are a federation v2 feature
- Only federation v2 subgraphs (with @link directive) should be considered for entity interface detection
- Federation v1 subgraphs should be ignored in this determination