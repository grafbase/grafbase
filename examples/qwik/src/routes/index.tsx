import type { RequestHandler } from '@builder.io/qwik-city'
import { Resource, component$ } from '@builder.io/qwik'
import { useEndpoint } from '@builder.io/qwik-city'

type Message = {
  id: string
  author: string
  message: string
  createdAt: string
}

type Data = {
  messageCollection: { edges: { node: Message }[] }
}

export const GetAllMessagesQuery = /* GraphQL */ `
  query GetAllMessages($first: Int!) {
    messageCollection(first: $first) {
      edges {
        node {
          id
          author
          message
          createdAt
        }
      }
    }
  }
`

export const onGet: RequestHandler<Data> = async () => {
  const res = await fetch(`http://127.0.0.1:4000/graphql`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
      // 'x-api-key': `Your token...`
    },
    body: JSON.stringify({
      query: GetAllMessagesQuery,
      variables: {
        first: 100
      }
    })
  })

  const { data } = await res.json()

  return data
}

export default component$(() => {
  const productData = useEndpoint<Data>()

  return (
    <Resource
      value={productData}
      onPending={() => <div>Loading...</div>}
      onRejected={() => <div>Error</div>}
      onResolved={({ messageCollection }) => (
        <>
          <h1>Grafbook</h1>
          <ul>
            {messageCollection?.edges?.map(({ node }) => (
              <li key={node.id}>
                <p>
                  <strong>
                    <span>{node.author}</span>
                    <br />
                    <small>
                      {new Intl.DateTimeFormat('en-GB', {
                        dateStyle: 'medium',
                        timeStyle: 'short'
                      }).format(Date.parse(node.createdAt))}
                    </small>
                  </strong>
                </p>
                <p>{node.message}</p>
              </li>
            ))}
          </ul>
        </>
      )}
    />
  )
})
