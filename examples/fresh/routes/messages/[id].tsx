import { Handlers, PageProps } from '$fresh/server.ts'
import { grafbaseClient } from '@/utils/grafbase'

type Message = {
  id: string
  author: string
  message: string
  createdAt: string
}

const GetMessageByIdQuery = /* GraphQL */ `
  query GetMessageById($id: ID!) {
    message(by: { id: $id }) {
      id
      author
      message
    }
  }
`

export const handler: Handlers = {
  async GET(_, ctx) {
    const { id } = ctx.params

    const response = await grafbaseClient({
      query: GetMessageByIdQuery,
      variables: {
        id
      }
    })

    if (!response.ok) {
      return ctx.render(null)
    }

    const { data } = await response.json()

    return ctx.render(data)
  }
}

export default function MessagePage({ data }: PageProps<{ message: Message }>) {
  return (
    <>
      <h1>{data.message.author}</h1>
      <p>{data.message.message}</p>
    </>
  )
}
