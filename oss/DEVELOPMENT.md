# Development

## Getting Started

### With cargo-make

#### Prerequisites

- [Rust](https://www.rust-lang.org/learn/get-started)
- [Node.js](https://nodejs.org)
- [`cargo-make`](https://github.com/sagiegurari/cargo-make#installation) <!-- TBD -->
- [`cargo-nextest`](https://nexte.st/book/installation.html)
- [`esbuild`](https://esbuild.github.io/getting-started/#install-esbuild)

### With Nix

**Note**: Internally this has been tested on aarch64-darwin and x86_64-linux. Other platforms may need tweaks in flake.nix

#### Prerequisites

- [Nix](https://nixos.org/download.html)
- [Activate flakes](https://nixos.wiki/wiki/Flakes)
- [direnv](https://github.com/direnv/direnv)
- [nix-direnv](https://github.com/nix-community/nix-direnv)

Run the following in the root of the reposiotry:

```sh
echo 'use flake\ndotenv_if_exists .env\n' >>.envrc
direnv allow .
```

**Note**: Your editor may need a direnv extension or to be run from the command line in the project directory tree to be able to use the project tooling

## Running the project

TBD

<!-- will be added once this moves and is a crate -->

## Testing

```sh
cargo nextest run
```

### E2E

TBD

## PRs

We require that all PRs titles follow a semantic PR scheme.

```
feat: Adds support for ray tracing
──┬─  ─────────────┬──────────────
  │                │
  │                │
  │                │
Prefix        Description
```

### Allowed Prefixes

| Prefix    | Description                                                                  |
| --------- | ---------------------------------------------------------------------------- |
| `chore`   | An internal change this is not observable by users                           |
| `enhance` | Any user observable enhancement to an existing feature                       |
| `feat`    | Any user observable new feature, such as a new connector, auth provider, etc |
| `fix`     | Any user observable bug fix                                                  |
| `docs`    | A documentation change                                                       |
| `revert`  | A revert of a previous change                                                |

## PR Checklist

- Have all Rust files been linted with `cargo clippy --locked --tests --all-targets -- -D warnings`?
- Have all Rust files been formatted with `rustfmt`?
- Have all non-Rust files been formatted with `prettier`?
- Are all tests passing?
- Does the PR have a detailed description of the change and the reasoning behind it?

## Issues

### Bugs

Please use the following template for bug reports:

```md
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:

**Expected behavior**
A clear and concise description of what you expected to happen.

**Examples**
If applicable, add examples of input and output to help explain your problem.

**Environment (please complete the following information)**

- Affected Component: [CLI, Gateway]
- Affected version: [e.g. 0.0.1]
- OS: [e.g. macOS]
- OS Version: [e.g. v0.11.0]

**Additional context**
Add any other context about the problem here.
```

### Feature Suggestions

```md
**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe the problem this feature would solve (if applicable)**
A clear and concise description of the problem that led you to request this issue

**Additional context**
Add any other context or screenshots about the feature request here.
```
