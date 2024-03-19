# trusted-documents-urql

This example demonstrates a basic setup with Trusted Documents using [urql](https://commerce.nearform.com/open-source/urql/) as a client.

Below is a list of steps you can follow to see a successful trusted documents query. We'll use [Bun](https://bun.sh/) in this walkthrough, but node + (p)npm should work as well.

1. Install the node dependencies:

```sh
$ bun install
```

2. Create a self-hosted graph. Publish the public [Star Wars API example](https://studio.apollographql.com/public/star-wars-swapi/home?variant=current) as one of the subgraphs. You can also use another subgraph, but you will have to update the `schema.graphql` file and the queries in the example.

3. Start the gateway. The `grafbase.toml` file is included next to this README.

```sh
$ GRAFBASE_ACCESS_TOKEN="<your-access-token>" grafbase-gateway --config grafbase.toml --graph-ref=<name-of-your-graph>@main
```

4. Generate the trusted documents manifest. This step is optional because the generated `persisted-query-manifest.json` is already included in this example, but you will need to run the command if you change or add GraphQL queries.

```
$ bun run generate-persisted-query-manifest
```

5. Trust the queries:

```sh
npx -y grafbase trust <your-graph-ref> --manifest persisted-query-manifest.json --client-name democlient
```

6. Run the example:

```sh
$ bun run src/index.ts
```

You should see a successful response. To check that trusted documents are actually enforced, you can try modifying one of the queries: it will be rejected.
