import { useSession, signIn, signOut } from 'next-auth/react'
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

export default function Home() {
  const [data, setData] = useState()
  const { data: session } = useSession()

  if (session) {
    return (
      <>
        Signed in
        <br />
        <button onClick={() => signOut()}>Sign out</button>
        <button onClick={() => fetchMessages().then(({ data }) => setData(data))}>
          Fetch messages with token
        </button>
        <pre>{JSON.stringify({ data }, null, 2)}</pre>
      </>
    )
  }

  return (
    <>
      Not signed in <br />
      <button onClick={() => signIn()}>Sign in</button>
    </>
  )
}
