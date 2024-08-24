import { GraphQLSchema, GraphQLObjectType, GraphQLString } from 'graphql'
import http from 'http'
import { createHandler as createSseHandler } from 'graphql-sse/lib/use/http'
import { createHandler } from 'graphql-http/lib/use/http'

/**
 * Construct a GraphQL schema and define the necessary resolvers.
 *
 * type Query {
 *   hello: String
 * }
 * type Subscription {
 *   greetings: String
 * }
 */
const schema = new GraphQLSchema({
  query: new GraphQLObjectType({
    name: 'Query',
    fields: {
      hello: {
        type: GraphQLString,
        resolve: () => 'world',
      },
    },
  }),
  subscription: new GraphQLObjectType({
    name: 'Subscription',
    fields: {
      greetings: {
        type: GraphQLString,
        subscribe: async function* () {
          for (const hi of ['Hi', 'Bonjour', 'Hola', 'Ciao', 'Zdravo']) {
            yield { greetings: hi }
          }
        },
      },
    },
  }),
})

// Create the GraphQL over SSE handler
const sseHandler = createSseHandler({ schema })
const handler = createHandler({ schema })

// Create an HTTP server using the handler on `/graphql`
const server = http.createServer((req, res) => {
  if (req.url.startsWith('/graphql')) {
    if (req.headers.accept.includes('text/event-stream')) {
      return sseHandler(req, res)
    } else {
      return handler(req, res)
    }
  }
  res.writeHead(404).end()
})

// Handle ^C
process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)

// Do graceful shutdown
function shutdown() {
  console.log('Graceful shutdown...')
  server.close(function () {
    console.log('Closed server')
  })
}

server.listen(4092)

console.log('Listening to port 4092')
