import { Handlers, PageProps } from '$fresh/server.ts'

const GetAllPostsQuery = /* GraphQL */ `
  query GetAllPosts($first: Int!) {
    postCollection(first: $first) {
      edges {
        node {
          id
          title
          slug
        }
      }
    }
  }
`

export const handler: Handlers = {
  async GET(_, ctx) {
    const response = await fetch(Deno.env.get('GRAFBASE_API_URL'), {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        'x-api-key': Deno.env.get('GRAFBASE_API_KEY')
      },
      body: JSON.stringify({
        query: GetAllPostsQuery,
        variables: {
          first: 100
        }
      })
    })

    if (!response.ok) {
      return ctx.render(null)
    }

    const { data } = await response.json()

    return ctx.render(data)
  }
}

export default function Home({ data }: PageProps) {
  return (
    <>
      <h1>Posts from Grafbase</h1>
      <ul>
        {data?.postCollection?.edges?.map(({ node }) => (
          <li key={node.id}>
            <a href={`/posts/${node.slug}`}>{node.title}</a>
          </li>
        ))}
      </ul>
    </>
  )
}
