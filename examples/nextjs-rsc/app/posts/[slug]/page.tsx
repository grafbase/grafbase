export const revalidate = 3600

import { graphql } from '../../../gql'
import { grafbase } from '../../../lib/grafbase'

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

  return (
    <>
      <h1>{post?.title}</h1>
      <pre>{JSON.stringify(post, null, 2)}</pre>
    </>
  )
}

export default Page
