import { graphql } from '../../../gql'
import { grafbase } from '../../../lib/grafbase'

export const revalidate = 3600

const GetPostBySlugDocument = graphql(/* GraphQL */ `
  query GetPostBySlug($slug: String!) {
    post(by: { slug: $slug }) {
      id
      title
      slug
    }
  }
`)

const Page = async ({ params }: { params: { slug: string } }) => {
  const { post } = await grafbase.request(GetPostBySlugDocument, {
    slug: params.slug
  })

  if (!post) {
    // optionally import notFound from next/navigation
    return <h1>404: Not Found</h1>
  }

  return (
    <>
      <h1>{post.title}</h1>
      <pre>{JSON.stringify(post, null, 2)}</pre>
    </>
  )
}

export default Page
