// export const revalidate = 3600

import { useState } from 'react'

const GetPostBySlugQuery = /* GraphQL */ `
  query GetPostBySlug($slug: String!) {
    post(by: { slug: $slug }) {
      id
      title
      slug
      views
    }
  }
`

const UpdatePostViewCountBySlug = /* GraphQL */ `
  mutation UpdatePostViewBySlug($slug: ID!, $views: Int) {
    postUpdate(id: $slug, input: { views: $views }) {
      post {
        id
        views
      }
    }
  }
`

const getPostBySlug = async ({ slug }: { slug: string }) => {
  const res = await fetch(process.env.NEXT_PUBLIC_GRAFBASE_API_URL!, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
      // 'x-api-key': '...'
    },
    body: JSON.stringify({
      query: GetPostBySlugQuery,
      variables: { slug }
    })
  })

  return res.json()
}

// const updateViewCount = async ({
//   slug,
//   views
// }: {
//   slug: string
//   views: number
// }) => {
//   const res = await fetch(process.env.NEXT_PUBLIC_GRAFBASE_API_URL!, {
//     method: 'POST',
//     headers: {
//       'Content-Type': 'application/json'
//       // 'x-api-key': '...'
//     },
//     body: JSON.stringify({
//       query: UpdatePostViewCountBySlug,
//       variables: { slug, views }
//     })
//   })

//   return res.json()
// }

// const CommentForm = () => {
//   const [name, setName] = useState('')
//   const [message, setMessage] = useState('')

//   const handleSubmit = async (event) => {}

//   return (
//     <form onSubmit={handleSubmit}>
//       <input
//         type="text"
//         name="name"
//         placeholder="Your name"
//         value={name}
//         onChange={({ target: { value } }) => setName(value)}
//       />
//       <textarea
//         name="message"
//         placeholder="Write a short message"
//         value={message}
//         onChange={({ target: { value } }) => setMessage(value)}
//       />
//       <button type="submit">Post</button>
//     </form>
//   )
// }

const Page = async ({ params }: { params: { slug: string } }) => {
  const { data } = await getPostBySlug({ slug: params.slug })

  return (
    <>
      <h1>{data?.post?.title}</h1>
      <p>Some content here</p>
      <hr />
      {/* <CommentForm /> */}
    </>
  )
}

export default Page
