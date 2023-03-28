import { useUser } from '@auth0/nextjs-auth0/client'
import { useState } from 'react'

const fetchToken = async () =>
  await fetch('/api/auth/token').then((res) => res.json())

const fetchMessages = async () => {
  const { token } = await fetchToken()

  return fetch(process.env.NEXT_PUBLIC_GRAFBASE_API_URL!, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`
    },
    body: JSON.stringify({
      query: /* GraphQL */ `
        {
          messageCollection(first: 100) {
            edges {
              node {
                id
              }
            }
          }
        }
      `
    })
  })
    .then((res) => res.json())
    .catch((err) => console.log(err))
}

export default function Index() {
  const [data, setData] = useState()
  const { user, error, isLoading } = useUser()

  if (isLoading) return <div>Loading...</div>
  if (error) return <div>{error.message}</div>

  if (user) {
    return (
      <>
        Signed in
        <br />
        <a href="/api/auth/logout">Logout</a>
        {/* <br />
        <button
          onClick={() => fetchMessages().then(({ data: res }) => setData(res))}
        >
          Fetch messages with token
        </button>
        <pre>{JSON.stringify({ data }, null, 2)}</pre> */}
      </>
    )
  }

  return <a href="/api/auth/login">Login</a>
}
