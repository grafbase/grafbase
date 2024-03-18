import { createClient, fetchExchange, cacheExchange } from 'urql';
import { persistedExchange } from '@urql/exchange-persisted';
import gql from 'graphql-tag'

const planetsQuery = gql`
  query Planets {
    allPlanets {
      totalCount
       edges {
         node {
            name
            climates
            population
         }
       }
    }
  }`;

const peopleQuery = gql`
  query People($count: Int!) {
    allPeople(first: $count) {
         edges {
           node {
             name
           }
         }
    }




  }
`;


const client = createClient({
  url: 'http://localhost:1234/graphql',
  exchanges: [
    cacheExchange,
    persistedExchange({
      enableForMutation: true,
      enforcePersistedQueries: true,
      generateHash: (query) => {
        throw new Error("got into generateHash")
      }
    }),
    fetchExchange,
  ],
});

async function main() {
  client.call(planetsQuery)
}

main().catch(console.error).then(() => process.exit(1))
