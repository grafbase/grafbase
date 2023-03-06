import type { CodegenConfig } from "@graphql-codegen/cli";

const url = process.env.GRAFBASE_API_URL as string;
const xApiKey = process.env.GRAFBASE_API_KEY as string;

const config: CodegenConfig = {
  schema: [
    {
      [url]: {
        headers: {
          "x-api-key": xApiKey,
        },
      },
    },
  ],
  generates: {
    "graphql/schema.ts": {
      plugins: [
        "typescript",
        "typescript-operations",
        "typescript-graphql-request",
      ],
      config: {
        rawRequest: true,
      },
    },
  },
  documents: "./graphql/documents/**/*.graphql",
};
export default config;
