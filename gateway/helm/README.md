# Grafbase Gateway Helm Chart

The README file provides instructions for using and releasing the Grafbase Gateway Helm Chart.
The repository is hosted on [GitHub's Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry) (ghcr.io).

## Usage

It is recommended to not use this chart as-is due to it pointing to the latest unstable version.

Instead, find the latest Grafbase Gateway version you want to use, and override the tag in your values.yaml:

```yaml
image:
  tag: <VERSION>
```

## Secret Management

### Doppler Integration

This chart supports integration with [Doppler](https://www.doppler.com/) via the [Doppler Kubernetes Operator](https://docs.doppler.com/docs/kubernetes-operator) for secure secret management.

#### Prerequisites

1. Install the Doppler Kubernetes Operator:
   ```bash
   kubectl apply -f https://github.com/DopplerHQ/kubernetes-operator/releases/latest/download/recommended.yaml
   ```

2. Create a Doppler service token and store it in a Kubernetes secret:
   ```bash
   kubectl create secret generic doppler-token-secret \
     --from-literal=serviceToken=dp.st.your-token-here \
     -n doppler-operator-system
   ```

#### Configuration

Enable Doppler integration in your `values.yaml`:

```yaml
doppler:
  enabled: true
  project: "your-project"
  config: "prd"
  secretName: "grafbase-secrets"  # optional, defaults to release name
  resyncSeconds: 60               # optional, how often to sync secrets
```

#### Example

```yaml
# values-production.yaml
replicaCount: 3

doppler:
  enabled: true
  project: "grafbase-app"
  config: "production"
  secretName: "grafbase-production-secrets"

gateway:
  externalSchema: true
  args:
    - --graph-ref
    - your-graph@main
```

The chart will automatically:
- Create a `DopplerSecret` resource to sync secrets from Doppler
- Mount the synced secrets as environment variables in the gateway container
- Handle secret updates and rotation automatically

#### Available Secrets

Your Doppler project should contain secrets like:
- `GRAFBASE_ACCESS_TOKEN` - Access token for Grafbase Cloud
- `DATABASE_URL` - Database connection string
- `CUSTOM_SECRET` - Any custom environment variables

## Releasing

The repository is configured with a GitHub Actions workflow triggered when a new tag is pushed, that automatically packages and pushes the Helm chart to the GitHub Container Registry.

### New Release

1. Update the version in the `Chart.yaml` file:

    ```yaml
    version: 0.X.Y
    ```

    Replace `0.X.Y` with the new version number.

2. Commit the changes

3. Run [this workflow](https://github.com/grafbase/grafbase/actions/workflows/build-gateway-chart.yml) to publish the Helm chart to the GitHub Container Registry.

### Manual

You should use this method only in emergencies or if the GitHub Actions workflow is not working.

1. In order to push a Helm chart to ghcr.io, first you need to authenticate with GitHub:

    Generate a token from GitHub → Developer Settings → Tokens.

    Ensure it has at least the following permissions:

    - ✅ repo (Access private repositories)
    - ✅ write:packages (If pushing to GitHub Packages)

    Authenticate with the GitHub Container Registry using your personal access token:

    ```bash
    echo $TOKEN | helm registry login ghcr.io --username <your_github_username> --password-stdin
    ```

    Replace `<your_github_username>` with your GitHub username and `$TOKEN` with your personal access token.

2. Package the Helm Chart:

    Navigate to the directory containing your Helm chart and package it:

    ```bash
    cd gateway/helm
    helm package .
    ```

3. Push the packaged Helm chart to the Helm repository:

    ```bash
    helm push gateway-0.X.Y.tgz oci://ghcr.io/grafbase/helm-charts
    ```
