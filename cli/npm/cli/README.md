# Grafbase

The Grafbase CLI

## Usage

```
The Grafbase command line interface

USAGE:
    grafbase [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    dev          Start the Grafbase local development server
    completions  Output completions for the chosen shell to use, write the output to the appropriate location for your shell
    init         Sets up the current or a new project for Grafbase
    login        Log in to your Grafbase account
    logout       Log out of your Grafbase account
    create       Set up and deploy a new project
    deploy       Deploy your project
    link         Connect a local project to a remote project
    unlink       Disconnect a local project from a remote project
    logs         Tail logs from a standalone graph
    start        Start Grafbase in production mode
    build        Build the Grafbase project in advance to avoid the resolver build step in the start command
    introspect   Introspect a graph and print its schema
    subgraphs    List subgraphs
    schema       Fetch a federated graph or a subgraph
    publish      Publish a subgraph schema
    check        Check a graph for validation, composition and breaking change errors
```

## Subcommand Flag and Argument Documentation

### `completions`

Output completions for the chosen shell. to use, write the output to the appropriate location for your shell

```
Usage: grafbase completions <COMMAND>

Commands:
  bash        Generate completions for bash
  fish        Generate completions for fish
  zsh         Generate completions for zsh
  elvish      Generate completions for elvish
  powershell  Generate completions for powershell
```

### `dev`

Start the Grafbase local development server

```
Usage: grafbase dev [OPTIONS]

Commands:
  -p, --port <PORT>                                                 Use a specific port
  -s, --search                                                      If a given port is unavailable, search for another
      --disable-watch                                               Do not listen for schema changes and reload
      --log-level-functions <FUNCTION_LOG_LEVEL>                    Log level to print from function invocations, defaults to 'log-level' [possible values: none, error, warn, info, debug]
      --log-level-graphql-operations <GRAPHQL_OPERATION_LOG_LEVEL>  Log level to print for GraphQL operations, defaults to 'log-level' [possible values: none, error, warn, info, debug]
      --log-level-fetch-requests <FETCH_REQUEST_LOG_LEVEL>          Log level to print for fetch requests, defaults to 'log-level' [possible values: none, error, warn, info, debug]
      --log-level <LOG_LEVEL>                                       Default log level to print [possible values: none, error, warn, info, debug]
```

### `init`

Sets up the current or a new project for Grafbase

```
Usage: grafbase init [OPTIONS] [NAME]

Arguments:
  [NAME]
          The name of the project to initialize

Options:
  -t, --template <TEMPLATE>
          The name or GitHub URL of the template to use for the new project

  -g, --graph <GRAPH>
          What graph type (federated or standalone) to initialize the project with

          Possible values:
          - federated: Creates a federated graph
          - single:    Creates a standalone graph
```

### `create`

Set up and deploy a new graph

```
Usage: grafbase create [OPTIONS]

Options:
  -n, --name <NAME>       The name to use for the new graph
  -a, --account <SLUG>    The slug of the account in which the new project should be created
```

### `introspect`

Introspect a graph and print its schema

```
    Usage: grafbase introspect [OPTIONS] [URL]

Arguments:
  [URL]  GraphQL URL to introspect

Options:
  -H, --header [<HEADER>...]  Add a header to the introspection request
      --dev                   Pass this argument to introspect the local project. --url and --dev cannot be used together
```

### `subgraphs`

List all subgraphs for a graph

```
Usage: grafbase subgraphs <GRAPH_REF>

Arguments:
  <GRAPH_REF>  Graph reference following the format "account/graph@branch"
```

### `schema`

Fetch a federated graph or a subgraph schema

```
    Usage: grafbase schema [OPTIONS] <GRAPH_REF>

Arguments:
  <GRAPH_REF>  Graph reference following the format "account/graph@branch"

Options:
      --name <SUBGRAPH_NAME>  The name of the subgraph to fetch. If this is left out, the federated graph is fetched
```

### `publish`

Publish a subgraph schema

```
Usage: grafbase publish [OPTIONS] --name <SUBGRAPH_NAME> --url <URL> <--dev|GRAPH_REF> <--dev|--schema <SCHEMA_PATH>>

Arguments:
  [GRAPH_REF]  Graph reference following the format "account/graph@branch"

Options:
      --dev                          Publish to a running development server
      --name <SUBGRAPH_NAME>         The name of the subgraph
      --schema <SCHEMA_PATH>         The path to the GraphQL schema file to publish. If this argument is not provided, the schema will be read from stdin
      --url <URL>                    The URL to the GraphQL endpoint
      --dev-api-port <DEV_API_PORT>  The listening port of the federated dev [default: 4000]
  -H, --header [<HEADER>...]         Add a header to the introspection request
```

### `check`

Check a graph for validation, composition and breaking change errors

```
Usage: grafbase check [OPTIONS] <GRAPH_REF>

Arguments:
  <GRAPH_REF>  Graph reference following the format "account/graph@branch"

Options:
      --name <SUBGRAPH_NAME>  The name of the subgraph to check. This argument is always required in a federated graph context, and it should not be used in a standalone graph context
      --schema <SCHEMA>       The path to the GraphQL schema to check. If this is not provided, the schema will be read from STDIN
```
