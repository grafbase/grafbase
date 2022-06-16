import type { ClientOptions } from "@urql/core/dist/types/client";
import { createClient } from "urql";

const urqlClientBaseConfig: ClientOptions = {
  url: "/api/graphql",
  requestPolicy: "cache-and-network",
};

export const urqlClient = createClient({
  ...urqlClientBaseConfig,
});
