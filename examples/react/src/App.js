import React, { useEffect, useState } from 'react';

const App = () => {
  const [data, setData] = useState(null);
  
  const GetAllMessagesQuery = /* GraphQL */ `
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

  useEffect(() => {
    const fetchData = async () => {
      const response = await fetch('http://localhost:4000/graphql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ query: GetAllMessagesQuery,
          variables: {
            first: 100
          } }),
      });

      const result = await response.json();
      console.log(result.data)
      setData(result.data);
    };

    fetchData();
  }, []);

  return (
    <div>
      <h3>Grafbase Messages</h3>
      {data && (
        <>
        <ul>
          {data.messageCollection?.edges?.map(({ node }) => (
            <li key={node.id}>{node.author} - {node.message} - {node.createdAt}</li>
          ))}
        </ul>
        </>
      )}
    </div>
  );
};

export default App;
