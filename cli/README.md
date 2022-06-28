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

Welcome to the Grafbase CLI directory!

This directory is a cargo workspace with several crates relating to the Grafbase CLI and developer tooling

The directories in the `crates` directory in the root of the repo are organized as follows:

| Directory                  | Description                             |
| -------------------------- | --------------------------------------- |
| [backend](crates/backend/) | Universal backend for Grafbase devtools |
| [cli](crates/cli/)         | Command Line Interface                  |
| [common](crates/common/)   | Shared functions and utilities          |
| [server](crates/server/)   | Wrapper for the API worker              |
