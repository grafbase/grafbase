import { graph, config } from "@grafbase/sdk";

export default config({
  graph: graph.Federated({
    headers: (headers) => {
      headers.set('x-api-key', 'dummy')
    }
  }),
});
