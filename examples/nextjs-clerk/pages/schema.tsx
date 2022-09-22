import { useAuth } from '@clerk/nextjs'
import { useState } from 'react'

const query = /* GraphQL */ `
  {
    postCollection(first: 100) {
      edges {
        node {
          id
          title
          comments {
            edges {
              node {
                id
                message
              }
            }
          }
        }
      }
    }
  }
`

const SchemaPage = () => {
  const [data, setData] = useState()
  const { getToken } = useAuth()

  const fetchData = async () => {
    await fetch(process.env.NEXT_PUBLIC_GRAFBASE_API_URL as string, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        authorization: `Bearer ${await getToken({
          template: 'grafbase'
        })}`
      },
      body: JSON.stringify({ query })
    }).then((res) => res.json().then(({ data }) => setData(data)))
  }

  return (
    <div>
      <button onClick={fetchData}>Fetch data</button>
      <pre>{JSON.stringify({ data }, null, 2)}</pre>
    </div>
  )
}

export default SchemaPage
