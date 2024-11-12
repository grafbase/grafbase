# Wasm component loader for the Grafbase Gateway

Adds support for loading WebAssembly components in the Grafbase Gateway. The Wasm file has to be in a form of WASI Preview 2 component, which can be created using the [cargo-component](https://github.com/bytecodealliance/cargo-component) tooling. See the examples for simple guest components, which are all tested in CI for this crate and work together with the host library.

The component interface must define the types defined in a wit file:

```wit
package component:grafbase;

interface types {
    enum header-error {
        invalid-header-value,
        invalid-header-name,
    }

    resource headers {
        get: func(name: string) -> result<option<string>, header-error>;
        set: func(name: string, value: string) -> result<_, header-error>;
        delete: func(name: string) -> result<option<string>, header-error>;
    }

    resource gateway-request {
        get-operation-name: func() -> option<string>;
        set-operation-name: func(name: option<string>);
        get-document-id: func() -> option<string>;
        set-document-id: func(id: option<string>);
    }

    record error-response {
        status: option<u16>,
        message: string,
    }
}

world gateway {
    use types.{headers, gateway-request, error-response};

    export on-gateway-request: func(headers: headers, request: gateway-request) -> result<_, error-response>;
}
```

The world defines the functions the guest is interested to plug into. If not wanting to handle this exact hook, the hook should be removed from the wit definition. If the host cannot find that exact function from the guest, the host hook will be a no-op.

Only tested with Rust guests so far. If using another guest language, your mileage may vary.
