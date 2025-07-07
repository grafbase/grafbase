# Improvements

- The subgraph owners are now shown for subgraphs that are loaded from the schema registry in the `grafbase dev` app.
- Previously, if the configuration file was not found, an empty configuration was assumed in all commands. This lead to commands like grafbase extension install returning with a successful exit status when no configuration is present. Since a failure would be expected in this case, the CLI should return with a non-zero exit status. This release makes extension commands fail on missing configuration. (https://github.com/grafbase/grafbase/pull/3269)
