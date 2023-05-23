import { revalidateTag } from 'next/cache'

import { gql } from './grafbase'

export default async function Home() {
  const query = /* GraphQL */ `
    {
      messageCollection(first: 100) {
        edges {
          node {
            id
            author
            body
            createdAt
          }
        }
      }
    }
  `

  const data = await gql(query)

  async function handleSubmit(formData: FormData) {
    'use server'

    const author = formData.get('author')
    const body = formData.get('body')

    if (!author || !body) {
      // throw new Error('Author and body is required')
    }

    await gql(/* GraphQL */ `
       mutation {
         messageCreate(input: {
           author: "${author}"
           body: "${body}"
         }) {
           message {
             id
           }
         }
       }
    `)

    revalidateTag(query)
  }

  return (
    <div className="px-6 py-12 max-w-xl mx-auto space-y-6">
      <h1 className="text-5xl font-bold">Grafbook</h1>
      <form action={handleSubmit} className="space-y-3">
        <input
          type="text"
          id="author"
          name="author"
          placeholder="Name"
          className="bg-gray-50 w-full block p-3 rounded"
        />
        <textarea
          id="body"
          name="body"
          placeholder="Write a message..."
          rows={5}
          className="bg-gray-50 w-full block p-3 rounded"
        ></textarea>
        <button
          type="submit"
          className="bg-green-500 text-white w-full p-3 rounded"
        >
          Submit
        </button>
      </form>
      <ul className="space-y-3">
        {data?.messageCollection?.edges?.map(({ node }: any) => (
          <li key={node.id} className="bg-gray-50 p-3 rounded space-y-3">
            <p className="flex items-center justify-between">
              <strong className="text-green-500">{node.author}</strong>
              <small className="text-gray-500 text-xs">
                {new Intl.DateTimeFormat('en-GB', {
                  dateStyle: 'medium',
                  timeStyle: 'short'
                }).format(Date.parse(node.createdAt))}
              </small>
            </p>
            <p className="text-gray-800">{node.body}</p>
          </li>
        ))}
      </ul>
    </div>
  )
}
