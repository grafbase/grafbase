import { Client, fetchExchange, cacheExchange, gql } from 'urql';
import { persistedExchange } from '@urql/exchange-persisted';
import manifest from '../persisted-query-manifest.json'

const queryMap = manifest.operations.reduce((acc, item) => {
  acc[item.body] = item.id;
  return acc
}, {} as { [key: string]: string })


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


const client = new Client({
  url: 'http://127.0.0.1:5000/graphql',
  fetchOptions: {
    headers: {
      'x-grafbase-client-name': 'democlient'
    }
  },
  exchanges: [
    cacheExchange,
    persistedExchange({
      enableForMutation: true,
      enforcePersistedQueries: true,
      generateHash: (query) => Promise.resolve(queryMap[query]),
    }),
    fetchExchange,
  ],
});

async function main() {
  const response = await client.query(peopleQuery, { count: 4 })
  console.log(JSON.stringify(response, null, 2))
}

main().catch(console.error).then(() => process.exit(1))
