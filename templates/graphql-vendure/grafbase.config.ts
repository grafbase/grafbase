import { graph, config, connector } from "@grafbase/sdk";

const g = graph.Standalone();

const vendureUrl =
  process.env.VENDURE_GRAPHQL_API_URL || "https://demo.vendure.io/shop-api";

const vendure = connector.GraphQL("Vendure", {
  url: vendureUrl,
});

g.datasource(vendure, { namespace: false });

export default config({
  graph: g,
  auth: {
    rules: (rules) => {
      rules.public();
    },
  },
  cache: {
    rules: [
      {
        types: ["Query"],
        maxAge: 60,
        staleWhileRevalidate: 60,
      },
    ],
  },
});
