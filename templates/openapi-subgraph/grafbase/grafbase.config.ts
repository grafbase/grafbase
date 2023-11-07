import { g, connector, config } from "@grafbase/sdk";

const openapi = connector.OpenAPI("OpenAPI", {
  schema: g.env("SCHEMA_URL"),
});

g.datasource(openapi, { namespace: false });

export default config({
  schema: g,
  auth: {
    rules: (rules) => {
      rules.public();
    },
  },
});
