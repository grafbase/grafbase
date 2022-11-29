import { useUser } from '@clerk/clerk-react'
import { useEffect, useState } from 'react'
import { useQuery, gql, useMutation } from 'urql'

type Message = {
  id: string
  author: string
  message: string
  createdAt: string
}

const GetAllMessagesQuery = gql`
  query GetAllMessages($first: Int!) @live {
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

const AddNewMessageMutation = gql`
  mutation AddNewMessage($author: String!, $message: String!) {
    messageCreate(input: { author: $author, message: $message }) {
      message {
        id
      }
    }
  }
`

function App() {
  const [{ data, fetching, error }] = useQuery<{
    messageCollection: { edges: { node: Message }[] }
  }>({
    query: GetAllMessagesQuery,
    variables: { first: 100 }
  })
  const [{}, addNewMessage] = useMutation(AddNewMessageMutation)

  const { user } = useUser()
  const [author, setAuthor] = useState('')
  const [message, setMessage] = useState('')
  useEffect(() => {
    if (user?.username) {
      setAuthor(user.username)
    }
  }, [])

  if (fetching) return <p>Loading...</p>
  if (error) return <p>Error : {error.message}</p>

  return (
    <>
      <h1>Grafbook</h1>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          if (author && message) {
            addNewMessage({ author, message })
            setMessage('')
          }
        }}
      >
        <fieldset>
          <legend>New message</legend>
          <input
            id="author"
            name="author"
            placeholder="Name"
            value={author}
            onChange={(e) => setAuthor(e.target.value)}
          />
          <br />
          <textarea
            id="message"
            name="message"
            placeholder="Write a message..."
            rows={5}
            value={message}
            onChange={(e) => setMessage(e.target.value)}
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
                <span>{node.author}</span>
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

export default App
