# JWT Authentication Extension

This is a proof of concept authentication extension for the Grafbase Gateway, implementing the native JWT authentication mechanism as a WebAssembly component.

## Installing

Until the Grafbase Extension Registry is done, you must build this extension manually and copy the artifacts to a place where the gateway can find them.

```bash
grafbase extension build
```

The resulting wasm component and a manifest file are generated in the `build` directory.

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
