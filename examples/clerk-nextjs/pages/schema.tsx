import { useAuth } from '@clerk/nextjs'
import useSWR from 'swr'

export const useQuery = (query: any, variables?: any) => {
  if (!query) {
    throw Error('No query provided to `useQuery`')
  }

  const { getToken } = useAuth()

  const fetcher = async () =>
    await fetch(process.env.NEXT_PUBLIC_GRAFBASE_URL as string, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        authorization: `Bearer ${await getToken({
          template: 'grafbase'
        })}`
      },
      body: JSON.stringify({ query, variables })
    }).then((res) => res.json().then(({ data }) => data))

  return useSWR(query, fetcher)
}

const SchemaPage = () => {
  const { data } = useQuery(`query { __schema { types { name } } }`)

  return (
    <h2>
      GraphQL schema has {data?.__schema?.types?.length || 0} types
      <pre>{JSON.stringify({ data }, null, 2)}</pre>
    </h2>
  )
}

export default SchemaPage
