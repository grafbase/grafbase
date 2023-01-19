import { component$, useSignal, useTask$ } from '@builder.io/qwik';


type Message = {
  id: string
  author: string
  message: string
  createdAt: string
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

export default component$(() => {
  const messages = useSignal<Message[]>([])

  useTask$(async () => {

    const response = await fetch(`http://127.0.0.1:4000/graphql`,
    { 
      method: 'POST',
      headers: {
      'content-type': 'application/json',
     // to do: 'authorization': `BEARER {token}`
      },
      body: JSON.stringify({ 
        query: GetAllMessagesQuery,
        variables: {
          first: 100
        } 
      }) 
    });
    messages.value = await response.json();
  });

  return (
    <div>
       <ul>
        {messages.value.data?.messageCollection?.edges?.map(({ node }) => (
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
    </div>
  );
});