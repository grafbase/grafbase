import { Handlers, PageProps } from '$fresh/server.ts'
import { grafbaseClient } from '@/utils/grafbase'

type Message = {
  id: string
  author: string
  message: string
  createdAt: string
}

const GetAllMessagesQuery = /* GraphQL */ `
  query GetAllMessages($first: Int!) {
    messageCollection(first: $first) {
      edges {
        node {
          id
          author
          message
          createdAt
        }
      }
    }
  }
`

const AddNewMessageMutation = /* GraphQL */ `
  mutation AddNewMessage($author: String!, $message: String!) {
    messageCreate(input: { author: $author, message: $message }) {
      message {
        id
      }
    }
  }
`

export const handler: Handlers = {
  async GET(_, ctx) {
    const response = await grafbaseClient({
      query: GetAllMessagesQuery,
      variables: {
        first: 100
      }
    })

    if (!response.ok) {
      return ctx.render(null)
    }

    const { data } = await response.json()

    return ctx.render(data)
  },
  async POST(req, ctx) {
    const formData = await req.formData()
    const json = Object.fromEntries(formData)

    await grafbaseClient({
      query: AddNewMessageMutation,
      variables: {
        author: json.author,
        message: json.message
      }
    })

    const response = await grafbaseClient({
      query: GetAllMessagesQuery,
      variables: {
        first: 100
      }
    })

    const { data } = await response.json()

    return ctx.render(data)
  }
}

export default function IndexPage({
  data
}: PageProps<{ messageCollection: { edges: { node: Message }[] } }>) {
  return (
    <>
      <h1>Grafbook</h1>
      <form method="POST">
        <fieldset>
          <legend>New message</legend>
          <input id="author" name="author" placeholder="Name" />
          <br />
          <textarea
            id="message"
            name="message"
            placeholder="Write a message..."
            rows={5}
          ></textarea>
          <br />
          <button type="submit">Submit</button>
        </fieldset>
      </form>
      <ul>
        {data?.messageCollection?.edges?.map(({ node }) => (
          <li key={node.id}>
            <p>
              <strong>
                <a href={`/messages/${node.id}`}>{node.author}</a>
                <br />
                <small>
                  {new Intl.DateTimeFormat('en-GB', {
                    dateStyle: 'medium',
                    timeStyle: 'short'
                  }).format(Date.parse(node.createdAt))}
                </small>
              </strong>
            </p>
            <p>{node.message}</p>
          </li>
        ))}
      </ul>
    </>
  )
}
