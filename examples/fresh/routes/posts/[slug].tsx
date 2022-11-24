import { Handlers, PageProps } from '$fresh/server.ts'

const GetPostBySlugQuery = /* GraphQL */ `
  query GetPostBySlug($slug: String!) {
    post(by: { slug: $slug }) {
      id
      title
    }
  }
`

export const handler: Handlers = {
  async GET(_, ctx) {
    const { slug } = ctx.params

    const response = await fetch(Deno.env.get('GRAFBASE_API_URL'), {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        'x-api-key': Deno.env.get('GRAFBASE_API_KEY')
      },
      body: JSON.stringify({
        query: GetPostBySlugQuery,
        variables: {
          slug
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

export default function PostPage({ data }: PageProps) {
  return <h1>{data.post.title}</h1>
}
