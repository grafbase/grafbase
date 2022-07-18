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
    completions    Output completions for the chosen shell
                       To use, write the output to the appropriate location for your shell
    dev            Run your grafbase project locally
    help           Print this message or the help of the given subcommand(s)
    init           Sets up the current or a new project for Grafbase
```

## Subcommands

### `completions <shell>`

Output completions for the chosen shell

### `dev`

Run your grafbase project locally

#### Flags

- `-p, --port <port>` - Use a specific port
- `-s, --search` - If a given port is unavailable, search for another

### `init`

Sets up the current or a new project for Grafbase

#### Arguments

- `[name]` - If supplied, creates a new project directory with the given name rather than setting up the current directory
