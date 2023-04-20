// Replace with fetch/EventSourceReconnecting
import { GraphQLClient } from "graphql-request";
import { jsonToGraphQLQuery } from "json-to-graphql-query";

export class GrafbaseClient {
  constructor(options: { url: string; apiKey?: string; authToken?: string }) {
    const client = new GraphQLClient(options.url, {
      headers: {
        // this should check for the env :) api keys only in dev mode or server
        ...(options?.apiKey && { "x-api-key": options?.apiKey }),
        ...(options?.authToken && {
          Authorization: `Bearer ${options?.authToken}`,
        }),
      },
    });

    // execute fn
    // object

    return new Proxy(this, {
      get(target, property) {
        console.log({ target, property });

        return (selectionSet: any) => {
          console.log(selectionSet);

          // Allow custom query name for later analytics tracking?
          const { args: __args, fields, live = false } = selectionSet;

          // Check fields for unions and

          // Add mutation support
          const operation = "query";

          const query = {
            [operation]: {
              //
              // ...(live && { __directives: { live: true } }),
              [property]: {
                __args,
                ...fields,
              },
            },
          };

          // Build our own or delegate to schema
          const queryAsJson = jsonToGraphQLQuery(query);

          // Allow passing additional per request headers?
          return client.request(
            queryAsJson,
            {},
            // Initialize EventSource above and handle
            live ? { "content-type": "text/event-stream" } : {}
          );
        };
      },
    });
  }
}
