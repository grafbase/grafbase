import { GraphQLClient } from "graphql-request";
import { getSdk } from "../graphql/schema";

const url = process.env.GRAFBASE_API_URL as string;
const xApiKey = process.env.GRAFBASE_API_KEY as string;

const client = new GraphQLClient(url, {
  headers: {
    "x-api-key": xApiKey,
  },
});

export default getSdk(client);
