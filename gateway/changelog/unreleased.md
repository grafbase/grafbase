## Changes

- Subgraph error extensions were previously put under the `upstream_extensions` key, now we just re-use the `extensions` as is and only add an error code if not present already.

## Improvements

- The gateway now transparently supports gzip, brotli, deflate, and zstd compression for subgraph responses. It will advertise support through the `Accept-Encoding` header. That does not apply to the body of requests from the gateway to the subgraph, only responses. If you need opt-in request body compression, please contact us and it will be added in short notice. (https://github.com/grafbase/grafbase/pull/2743)

## Fixes

- fixed HTTP trace export with improved telemetry configuration handling, better merging global & tracing exporter parameters.
