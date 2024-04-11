# The Grafbase Gateway

Provides an HTTP server together with the Grafbase GraphQL engine to host federated graphs.

## Testing

The `gateway-binary` crate has tests for testing the command itself and its HTTP server. In the `federated-server` crate there are more fine-grained unit tests for config parsing and all that.

## Releasing

First create a new branch, call it e.g. `release-X.Y.Z`. Having a clean worktree in git, first bump the version:

```bash
> cargo make bump 1.0.0
```

This changes all the versions related to the gateway and creates a new changelog file. For a release, the new changelog file should include three sections, if applicable:

```markdown
### Features

- A new shiny thing

### Bugfixes

- Fixed a race condition

### Breaking

- This will break your server if you do not change your configuration or code, so be aware.
```

Small bugfixes should trigger a _patch_ release, where the last number changes. E.g. from `1.0.0` to `1.0.1`.

New features, that are not breaking, will change the second number (from `1.0.0` to `1.1.0`).

Breaking features should change the first number (from `1.0.0` to `2.0.0`).

If wanting to test a new feature with the users, but not yet do a proper release, a release candidate can be released, e.g. `1.0.0-rc.1` would be the first release candidate for the `1.0.0` version.

When a release commit is successfully done, push the branch, create a PR and wait for the CI to release. When the CI is green, merge the PR, fetch the latest `main` branch to your computer and start the release:

```bash
> cargo make release
```
