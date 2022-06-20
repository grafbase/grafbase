import type { ClientOptions } from "@urql/core/dist/types/client";
import {
  cacheExchange,
  createClient,
  dedupExchange,
  fetchExchange,
} from "urql";
import { errorExchange } from "./uqrl.error";

const urqlClientBaseConfig: ClientOptions = {
  url: "/api/graphql",
  requestPolicy: "cache-and-network",
};

export const urqlClient = createClient({
  ...urqlClientBaseConfig,
  exchanges: [dedupExchange, cacheExchange, errorExchange(), fetchExchange],
});
