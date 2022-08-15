# Changelog

## [0.5.0]

### Features

- Adds support for the `DateTime` scalar
- Adds a `reset` command (removes local data for a project)

### Tooling

- Updates Rust to `1.63.0`
- Bundles and minifies some of the embedded assets
  - slightly reduces overall size and solves an incompatibility issue in wasm-pack with node

## [0.4.1]

### Misc.

- Adds styling for the GraphQL playground

## [0.4.0]

### Breaking

- Supports new ID format (e.g. `author_01GA1B6QD2189C2GNQC7KNJRP2`)
- Supports `@unique` directive

### Features

- Differentiates and correctly reports user and logic errors (bugs) in SQL operations

### Testing

- Adds new tests

### Notes

- As this is a breaking change, if you have an existing database in a project (under `project/.grafbase`), please remove the `.grafbase` folder before running the CLI

## [0.3.0]

### Features

- Adds live reloading (`dev -w --watch`), reloads the development server when schema changes are detected
- Updates the default schema
- Adds polling support for the playground
- Adds `gb` as an alias for `grafbase`

### Fixes

- Detects creation events when watching the schema
- Opts out of watching rather than opt in

### Refactoring

- Allows passing a function returning any type to the file watcher
- Allows to skip file extraction in CI
- Removes `chrono` temporarily as it is optional

### Testing

- Adds cross platform integration tests

### Tooling

- Consolidates the CLI CI
- Allows skipping asset export using an env variable (for CI)
- Improves caching
- Exports assets to home folder in CI to reduce wait time
- Updates Rust to `1.62.1`

## [0.2.2]

### Dependencies

- Remove `detect-libc` from the NPM package as it's not currently in use

## [0.2.1]

### Fixes

- Fixes binary permissions (NPM only)

## [0.2.0]

- Beta release! 🎉

## [0.1.0-pre.7]

## Features

- Outputs playground and endpoint links on `dev`

## Fixes

- Exports embedded files when downgrading a version as well as upgrading
- Updates the default schema to add a field other than id

## [0.1.0-pre.6]

### Tooling

- Adds NPM distrbution mechanism

### Dependinces

- Normalizes `backtrace` version

## [0.1.0-pre.5]

### Features

- Adds a hint when `init` is run in an initialized project

### Fixes

- Fixes the CLI header being printed when printing completions

### Refactoring

- Cleans up the CLI `main.rs` file
- Moves the default GraphQL schema to a file

## [0.1.0-pre.4]

- Updates the worker, fixing an issue with deletion

## [0.1.0-pre.3]

- Removes the `v` from version output
- Shortens Node.js error hints

## [0.1.0-pre.2]

### Fixes

- Fixes the `completions` shell argument not being required

## [0.1.0-pre.1]

### Fixes

- Includes the assets when publishing to crates.io

## [0.1.0-pre.0]

### Features

- Initial partial implementation of the `dev` command
- Adds proper exit codes
- Implements port searching
- Implements an environment config
- Adds support for CLI completions
- Adds future arguments (commented out)
- Adds proper errors
- Allows supplying a specific port
- `dev` command phase 2
- Adds `miniflare` spawning
- Bundles worker files into the executable
- Adds an SQLite bridge server
- Stores the DB in the project `.grafbase` folder
- Allows searching the entire port range for an open port
- Adds panic hook
- Supports schema parsing
- Silences Miniflare output
- Adds `pk` and `sk` to DB modeling and indexes both
- Generates `.gitignore` files for cache dirs
- Prints CLI header
- Adds `--trace` flag
- Adds `gsi...` fields to db schema
- Sets all DB fields as non nullable
- Detects Node.js installation and version and correctly notifies the user of an issue
- Runs miniflare offline
- Passes bridge port to worker
- Exits on panic with a correct exit code
- Prevents CTRL+C from being detected as an abnormal shutdown when running the server
- Adds a goodbye message
- Reports the minimal supported Node.js version when erroring due to unsupported versions
- Compresses included assets
- Aligns the minimal supported Node.js version to the one miniflare uses
- Uses a specific bundled version of miniflare
- Compresses assets
- Sets miniflare to only listen locally
- Waits for bridge server to be ready before spawning miniflare
- Adds a basic implementation for `init` (disabled)
- Uncomments `init` command
- Reports API errors
- Updates `init` command description

### Fixes

- Checks the correct address for ports (`0.0.0.0` vs `127.0.0.1`)
- Does not report `miniflare error` automatically when the spawned thread returns an error
- Handles non utf-8 path errors
- Allows output of completions even when not in Grafbase project
- Doesn't initialize environment for `init` and `completions`
- Fixes an issue with the detection of unavailable ports

### Tooling

- Refactors the project into a workspace
  - Adds new crates - `cli`, `common`, `dev-server`, `local-gateway`
- Adds a `.cargo/config.toml`
- Adds an `.editorconfig`
- Adds a `.github/CODEOWNERS`
- Adds a `.github/ISSUE_TEMPLATE/bug_report.md`
- Adds a `.github/ISSUE_TEMPLATE/feature_request.md`
- Adds CI workflows (lint, build, test) and a few non enabled workflows (Miri, coverage)
- Improves the `.gitignore`
- Adds a `prettier` config and an ignore file
- Adds a `CHANGELOG.md`
- Adds a `CODE_OF_CONDUCT.md`
- Adds a `CONTRIBUTING.md`
- Adds a `PRIVACY.md`
- Adds minimal tracing
- Forbids unsafe code
- Improves `renovate.json`
- Adds `rust-toolchain.toml`
- Adds `rustfmt.toml`
- `.gitignore` cleanup
- Updates @actions/checkout to v3
- Adds hints for error messages
- Uses the worker `wrangler.toml` file for env variables
- Turns on pedantic linting
- Adds additional tracing
- Strips symbols from the release binary and runs some size optimizations
- Updates Rust version to `1.62`

### Refactoring

- Adds Clippy fixes
- Expands `Environment` with additional fields
- Lifts `tokio` one level up to simplify task spawning and handling
- Moves `.grafbase` dir creation to `servers`
- Adds `colorize!` macro
- Moves asset folder to root
- Removes deprecated `clap` APIs
- Manually instantiates `tokio` runtime due to a Rust Analyzer issue
- Uses only `node` rather than `npx` as well
- Automatically creates needed directories when exporting assets
- Prefixes internal crates
- Folds `colorize` and `panic-hook`
- Renames `colorize` to `watercolor`
- Renames `local-gateway` to `backend`
- Clippy fixes
- Preparation for new transactions
  - SQLite schema
    - Changes `type` to `entity_type`
    - Adds `relation_names`
    - Unquotes columns
- Removes the `init` `--template` flag as it's not implemented yet

### Documentation

- Improves the `README.md`
- Adds inline documentation
- Adds crate documentation

### Dependencies

- Updates `once_cell` to 1.12.0

### Misc.

- Updates PR template
- Updates README.md
