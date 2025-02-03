In this directory run a HTTP server that will serve `greet.json`

```bash
python3 -m http.server 8080
```

Compile the extension with grafbase. Use the most recent CLI version, you'll likely need to build it from outside this directory because of workspaces.

```
grafbase extension build
```

Run the server with the following. You'll need to change the hardcoded paths. I'm not really sure how we should manage that yet.

```bash
cargo run -p grafbase-gateway -- --config extensions/rest/example/grafbase.toml --schema extensions/rest/example/schema.graphql
```

Execute a HTTP request:

```bash
curl 'http://127.0.0.1:5000/graphql' --data '{"query":"query { greeting }"}' -H 'Content-Type: application/json'
```
