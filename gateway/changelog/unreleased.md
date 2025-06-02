## Breaking changes

- The optional service that exposes federated graphs and trusted documents to the gateway, previously called GDN, has a new implementation that uses different paths for the assets. This change is only relevant if:
  1. You use the self-hosted Enterprise Platform (in which case you should upgrade to 0.7.1 when you upgrade to this version of the gateway)
  1. You define a different endpoint for it using the `GRAFBASE_GDN_URL` environment variable. In which case, use the `GRAFBASE_OBJECT_STORAGE_URL` environment variable from this version up.
