# JWT Extension

This is a proof of concept authentication extension for the Grafbase Gateway that implements the native JWT authentication mechanism as a WebAssembly component.

## Installing

Add the following to your gateway configuration ("grafbase.toml"):

```toml
[extensions.jwt]
version = "0.1"
```

Then run `grafbase extension install`. The extension will be installed in the `grafbase_extensions` directory. That directory must be present when the gateway is started.

## Building from source

Build this extension manually and copy the artifacts to a location where the gateway can find them.

```bash
grafbase extension build
```

The `build` directory contains the resulting wasm component and manifest file.

```bash
build/
├── extension.wasm
└── manifest.json
```

In your gateway configuration, you can now load the extension from the `build` directory.

```toml
[extensions.jwt]
path = "/path/to/build"
```

## Configuration

This extension acts as an authentication provider for the Grafbase Gateway. After adding it to the extensions section, configure it as an authentication provider.

```toml
[[authentication.providers]]

[authentication.providers.extension]
extension = "jwt"

[authentication.providers.extension.config]
url = "https://example.com/.well-known/jwks.json"
issuer = "example.com"
audience = "my-project"
poll_interval = 60
header_name = "Authorization"
header_value_prefix = "Bearer "
```

## Testing

Compile the CLI and gateway binaries first in the workspace root:

```bash
cargo build -p grafbase -p grafbase-gateway
```

Start the needed docker services in the `extensions` directory:

```bash
docker compose up -d
```

Run the tests in this directory:

```bash
cargo test
```
