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

### Documentation

- Improves the `README.md`
- Adds inline documentation
