import React, { useEffect, useState } from 'react'

function App() {
  const [data, setData] = useState(null)

  const GetAllMessagesQuery = /* GraphQL */ `
    query GetAllMessages($first: Int!) {
      messageCollection(first: $first) {
        edges {
          node {
            id
            author
            body
            createdAt
          }
        }
      }
    }
  `

  useEffect(() => {
    const fetchData = async () => {
      const response = await fetch('http://localhost:4000/graphql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          query: GetAllMessagesQuery,
          variables: {
            first: 100
          }
        })
      })

      const result = await response.json()
      setData(result)
    }

    fetchData()
  })

  return (
    <div>
      <h3>Grafbase Messages</h3>
      {data && (
        <>
          <ul>
            {data.data.messageCollection?.edges?.map(({ node }) => (
              <li key={node.id}>
                {node.author} - {node.body} - {node.createdAt}
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  )
}

export default App
