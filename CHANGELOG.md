# Changelog

## [next]

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

### Refactoring

- Adds Clippy fixes

### Documentation

- Improves the `README.md`
- Adds inline documentation
- Adds crate documentation

### Dependencies

- Updates `once_cell` to 1.12.0

### Misc.

- Updates PR template
