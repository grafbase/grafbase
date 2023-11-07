import { g, connector, config } from "@grafbase/sdk";

const openapi = connector.OpenAPI("OpenAPI", {
  schema: g.env("SCHEMA_URL"),
});

g.datasource(openapi, { namespace: false });

export default config({
  schema: g,
  federation: { version: "2.3" },
  auth: {
    rules: (rules) => {
      rules.public();
    },
  },
});
