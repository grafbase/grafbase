<div align="center">
  <img alt="Grafbase Logo" height="250px" src="https://grafbase.com/images/other/grafbase-logo-circle.png">
  <h1><code>grafbase cli</code></h1>
  <a href="https://github.com/grafbase/cli/actions/workflows/build.yml"><img alt="build status" src="https://github.com/grafbase/cli/actions/workflows/build.yml/badge.svg"></a>
</div>

Welcome to the Grafbase CLI monorepo!

This repository is a cargo workspace with several crates relating to the Grafbase CLI and developer tooling

The directories in the `crates` directory in the root of the repo are organized as follows:

| Directory                              | Description                             |
| -------------------------------------- | --------------------------------------- |
| [cli](crates/cli/)                     | Command Line Interface                  |
| [local-gateway](crates/local-gateway/) | Universal backend for Grafbase devtools |
| [dev-server](crates/dev-server/)       | Wrapper for the API worker              |
| [common](crates/common/)               | Shared functions and utilities          |
