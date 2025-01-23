# Grafbase Gateway Helm Chart

Helm chart for installing Grafbase Gateway.

## Usage

It is recommended to not use this chart as-is due to it pointing to the latest unstable version.

Instead, find the latest Grafbase Gateway version you want to use, and override the tag in your values.yaml:

```yaml
image:
  tag: <VERSION>
```

## Releasing

```bash
helm package .
helm push <path_to_tgz>
```
