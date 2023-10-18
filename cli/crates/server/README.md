# Server

## Building assets

This crate includes non-rust assets. We are in the process of migrating the
process to assemble the final `assets.tar.gz` embedded in the CLI binary to
this repository.

If you see errors related to the assets, please use the following sections to
make sure the assets necessary to build the CLI are present.

### Pathfinder

If you see an error about not finding a bundled Pathfinder build to include in
the CLI binary, you must either provide the
`GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH` environment variable or build the cli-app
in this repository at the default location (`/packages/cli-app`).

You can run `cargo make build-cli-app` to build the Pathfinder wrapper used by
the CLI. This part of the CLI build should now succeed.

### assets.tar.gz

If you see a an error about including `assets.tar.gz`, you need to download a
built version of this file. See the CLI [`Makefile.toml`](../../Makefile.toml).
