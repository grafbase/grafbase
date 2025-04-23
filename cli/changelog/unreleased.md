## Features

- Introducing a new `grafbase compose` command to compose a federated graph from subgraph schemas. The input subgraphs are discovered in the same way as in `grafbase dev`, that is to say through the `subgraphs.$subgraph_name.schema_path` and `subgraph.$subgraph_name.introspection_url` configuration options, as well as the `--graph-ref` argument.

## Fixes

- Fixed infinite loops in file watcher causing `grafbase dev` to become unresponsive
