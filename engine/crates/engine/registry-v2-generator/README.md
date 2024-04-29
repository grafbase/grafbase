A generator for much of the code in registry-v2. When you run this it'll read
the graphql in domain and update the corresponding files in registry-v2.

### Running

Run this from the root of the repo:

```sh
cargo run -p registry-v2-generator && cargo fmt
```

It will write directly to the registry-v2 codebase.
