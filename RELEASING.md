# Releasing the gateway (Pantheon fork)

The Guild has ended maintenance of Grafbase, so `pantheon-systems/grafbase` maintains its own
fork. This document describes how to cut a **gateway** release from the fork. Releases publish two
artifacts to Pantheon-owned infrastructure:

- the gateway Docker image → `ghcr.io/pantheon-systems/grafbase-gateway`
- the gateway Helm chart → `ghcr.io/pantheon-systems/grafbase-helm-charts`

plus a GitHub Release carrying the built gateway binaries (linux musl x86_64/aarch64, aarch64
darwin, and the two lambda binaries).

> **Scope:** the fork only releases the gateway. The `grafbase` CLI (npm packages, the
> `grafbase-*` tag path, and `downloads.grafbase.com` CLI assets) is intentionally not released.
> The source still builds and lives in git; re-add the CLI release path if a consumer appears.

## One-time repository setup

Configure these in **Settings → Secrets and variables → Actions** on
`pantheon-systems/grafbase`.

### Repository variables

| Variable | Value | Used by |
|---|---|---|
| `DEPOT_PROJECT` | The Pantheon-owned Depot project ID | `binaries.yml`, `rust-prs.yml` |
| `GATEWAY_IMAGE` | *(optional)* overrides the gateway image; defaults to `ghcr.io/pantheon-systems/grafbase-gateway` | `binaries.yml` |
| `HELM_REGISTRY_REPOSITORY` | *(optional)* overrides the Helm OCI repo; defaults to `ghcr.io/pantheon-systems/grafbase-helm-charts` | `build-gateway-chart.yml` |

### Repository secrets

| Secret | Required? | Notes |
|---|---|---|
| `DEPOT_TOKEN` | Yes (while CI runs on Depot) | Project token for the Pantheon Depot project. CI (both PR checks and release builds) runs on `depot-*` runners. |
| `GITHUB_TOKEN` | Automatic | Provided by Actions; used to push images/charts to the org's GHCR. Ensure the repo grants `packages: write` (already declared in the workflows). |

`NPM_ACCESS_TOKEN`, `CRATES_ACCESS_TOKEN`, `DOCKERHUB_USERNAME/TOKEN`, and the Slack webhooks are
**no longer required** for gateway releases.

> **Runner note:** the entire CI (PR checks in `rust-prs.yml` and the release builds in
> `binaries.yml`) is built on Depot — including `depot/bake-action` for the docker-compose test
> stacks. That is why the release path keeps Depot and only un-hardcodes the project via
> `DEPOT_PROJECT`. Migrating to GitHub-hosted runners is a separate, larger effort.

## Cutting a release

1. **Bump the version.** Edit `version` in `gateway/Cargo.toml`, then update the lockfile:

   ```sh
   cargo update -p grafbase-gateway --precise <version>   # or: cargo build -p grafbase-gateway
   ```

2. **Write the changelog.** Add `gateway/changelog/<version>.md`. This file is **required** — the
   GitHub Release step reads `body_path: gateway/changelog/<version>.md` and fails if it is
   missing.

3. **Commit** on a branch and open a PR:

   ```sh
   git checkout -b release/gateway-<version>
   git commit -am "chore(release): gateway <version>"
   ```

   Merge it once CI is green.

4. **Tag and push** from the merged commit on `main`:

   ```sh
   git tag gateway-<version>
   git push origin gateway-<version>
   ```

   The `gateway-*` tag triggers `binaries.yml`, which builds the binaries + Docker image and then
   calls `partial-gateway-release.yml` to create the GitHub Release.

5. **Publish the Helm chart.** After bumping `version` in `gateway/helm/Chart.yaml`, run the
   **Gateway Helm Chart** workflow (`build-gateway-chart.yml`) via **Actions → Run workflow**
   (`workflow_dispatch`). It packages `gateway/helm` and pushes to `HELM_REGISTRY_REPOSITORY`.

## Prereleases

Tags containing `alpha`, `beta`, or `rc` are marked as GitHub prereleases automatically
(e.g. `gateway-<version>-rc.1`). Use one to smoke-test the whole path end-to-end before a real
release: confirm the binaries build, the image lands in `ghcr.io/pantheon-systems/grafbase-gateway`,
and a prerelease GitHub Release is created — then dispatch the Helm workflow.

## What runs on a `gateway-*` tag

`binaries.yml`:

- `check` → reuses `rust-prs.yml`
- `docker-gateway` → builds & pushes the multi-arch image (`linux/amd64,linux/arm64`)
- `linux` / `darwin` → build the gateway binaries + lambda binaries, upload as artifacts
- `release-gateway` → `partial-gateway-release.yml` downloads the artifacts, generates
  `licenses.html`, and creates the GitHub Release

## Follow-ups / not done here

- The PR-only `docker-gateway`/`examples` jobs in `rust-prs.yml` still reference
  `ghcr.io/grafbase/gateway`; they are gated behind `DOCKERHUB_USERNAME` and are inert without it,
  so they are not release-blocking. Retargeting them also requires updating the `examples/*`
  Dockerfiles and is left as a follow-up.
- Automated version-bump/changelog (e.g. a `workflow_dispatch` job or `cargo-release`) can replace
  the manual steps above later.
