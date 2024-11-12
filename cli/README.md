<p align="center">
  <a href="https://grafbase.com">
    <img src="https://grafbase.com/images/other/grafbase-logo-circle.png" height="96">
  </a>
  <h3 align="center">Grafbase CLI</h3>
</p>

<p align="center">
  <a href="https://github.com/grafbase/grafbase/actions/workflows/cli-build.yml">
    <img alt="CLI build status" src=https://github.com/grafbase/grafbase/actions/workflows/cli-build.yml/badge.svg>
  </a>
</p>

## Usage

```
The Grafbase command line interface

Usage: grafbase [OPTIONS] <COMMAND>

Commands:
  branch       Graph branch management
  completions  Output completions for the chosen shell to use, write the output to the appropriate
               location for your shell
  login        Logs into your Grafbase account
  logout       Logs out of your Grafbase account
  create       Set up and deploy a new graph
  introspect   Introspect a graph and print its schema
  subgraphs    List subgraphs
  schema       Fetch a federated graph or a subgraph
  publish      Publish a subgraph schema
  check        Check a graph for validation, composition and breaking change errors
  trust        Submit a trusted documents manifest
  lint         Lint a GraphQL schema
  help         Print this message or the help of the given subcommand(s)

Options:
  -t, --trace <TRACE>  Set the tracing level [default: 0]
      --home <HOME>    An optional replacement path for the home directory
  -h, --help           Print help
  -V, --version        Print version
```
