import { component$, useSignal } from '@builder.io/qwik'

type Message = {
  id: string
  author: string
  message: string
  createdAt: string
}

export const GetAllMessagesQuery = /* GraphQL */ `
  query GetAllMessages($first: Int!) @live {
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

export default component$(async () => {
  const messages = useSignal<Message[]>([])

  const {
    data: { messageCollection }
  }: {
    data: { messageCollection: { edges: { node: Message }[] } }
  } = await fetch(import.meta.env.VITE_GRAFBASE_API_URL as string, {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify({
      query: GetAllMessagesQuery,
      variables: {
        first: 100
      }
    })
  }).then((res) => res.json())

  return (
    <div>
      <h1>Messages</h1>
      {messages.values?.map(({ node }) => (
        <>
          <div>
            {node?.name} : {node?.description}
          </div>
        </>
      ))}

    </div>
  )
})
