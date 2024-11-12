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

USAGE:
    grafbase [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    completions    Output completions for the chosen shell
                       To use, write the output to the appropriate location for your shell
    dev            Start the Grafbase local development server
    help           Print this message or the help of the given subcommand(s)
    init           Sets up the current or a new project for Grafbase
```

## Subcommands

### `completions <shell>`

Output completions for the chosen shell

### `dev`

Start the Grafbase local development server

#### Flags

- `-p, --port <port>` - Use a specific port
- `-s, --search` - If a given port is unavailable, search for another
- `--disable-watch` - Do not listen for schema changes and reload

### `init`

Sets up the current or a new project for Grafbase

#### Arguments

- `[name]` - If supplied, creates a new project directory with the given name rather than setting up the current directory

#### Flags

- `-t, --template <name>` - The name or GitHub URL of the template to use for the new project
