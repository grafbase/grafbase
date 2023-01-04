import { Environment, Network, RecordSource, Store } from 'relay-runtime'

async function fetchRelay(params, variables) {
  return fetch('http://localhost:4000/graphql', {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify({
      query: params.text,
      variables
    })
  }).then((res) => res.json())
}

export default new Environment({
  network: Network.create(fetchRelay),
  store: new Store(new RecordSource())
})
