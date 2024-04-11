import { graph, config } from "@grafbase/sdk";

export default config({
  graph: graph.Federated({
    subgraphs: [
      { name: "accounts", url: "http://localhost:4001/query" },
      { name: "products", url: "http://localhost:4002/query" },
      { name: "reviews", url: "http://localhost:4003/query" },
    ],
  }),
});
