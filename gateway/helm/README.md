# Grafbase Gateway Helm Chart

The README file provides instructions for using and releasing the Grafbase Gateway Helm Chart.
The repository is hosted on [GitHub's Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry) (ghcr.io).

## What is ghcr.io?

ghcr.io allows you to store and manage Docker container images and Helm charts within your GitHub account.
It provides a secure and scalable way to distribute your containerized applications and Helm charts.

## Usage

It is recommended to not use this chart as-is due to it pointing to the latest unstable version.

Instead, find the latest Grafbase Gateway version you want to use, and override the tag in your values.yaml:

```yaml
image:
  tag: <VERSION>
```

## Releasing

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

### GitHub Actions

The repository is configured with a GitHub Actions workflow triggered when a new tag is pushed, that automatically packages and pushes the Helm chart to the GitHub Container Registry.
