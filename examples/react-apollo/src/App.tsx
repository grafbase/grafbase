import { useEffect, useState } from 'react'
import { gql, useQuery, useMutation } from '@apollo/client'
import { SignJWT } from 'jose'

type Message = {
  id: string
  author: string
  body: string
  createdAt: string
}

const GetAllMessagesQuery = gql`
  query GetAllMessages($first: Int!) {
    messageCollection(first: $first) {
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

const AddNewMessageMutation = gql`
  mutation AddNewMessage($author: String!, $body: String!) {
    messageCreate(input: { author: $author, body: $body }) {
      message {
        id
      }
    }
  }
`

const DeleteMessageMutation = gql`
  mutation DeleteMessage($id: ID!) {
    messageDelete(by: { id: $id }) {
      deletedId
    }
  }
`

// Do this on the server
const secret = new Uint8Array(
  (process.env.REACT_APP_JWT_SECRET as string)
    .split('')
    .map((c) => c.charCodeAt(0))
)

const getToken = (role: string) => {
  const groups = role ? [role] : []
  return new SignJWT({ sub: 'user_1234', groups })
    .setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
    .setIssuer(process.env.REACT_APP_ISSUER_URL as string)
    .setIssuedAt()
    .setExpirationTime('2h')
    .sign(secret)
}

function App() {
  const [role, setRole] = useState('')
  const { data, loading, error } = useQuery<{
    messageCollection: { edges: { node: Message }[] }
  }>(GetAllMessagesQuery, {
    variables: { first: 100 }
  })
  const [addNewMessage, { error: addNewMessageError }] = useMutation(
    AddNewMessageMutation
  )
  const [deleteMessage, { error: deleteMessageError }] = useMutation(
    DeleteMessageMutation
  )

  const [author, setAuthor] = useState('')
  const [body, setBody] = useState('')

  useEffect(() => {
    const setToken = async () => {
      const token = await getToken(role)
      localStorage.setItem('token', token)
    }
    setToken()
  }, [role])

  if (loading) return <p>Loading...</p>
  if (error) return <p>Error : {error.message}</p>

  return (
    <>
      <h1>Grafbook</h1>
      <div>
        <button onClick={() => setRole('')} disabled={role === ''}>
          Set role to user (read only)
        </button>{' '}
        <button onClick={() => setRole('admin')} disabled={role === 'admin'}>
          Set role to admin
        </button>
      </div>
      {!!addNewMessageError && <pre>Error : {addNewMessageError.message}</pre>}
      {!!deleteMessageError && <pre>Error : {deleteMessageError.message}</pre>}
      <br />
      <form
        onSubmit={(e) => {
          e.preventDefault()
          if (author && body) {
            addNewMessage({ variables: { author, body } })
            setBody('')
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
            id="body"
            name="body"
            placeholder="Write a message..."
            rows={5}
            value={body}
            onChange={(e) => setBody(e.target.value)}
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
            <p>{node.body}</p>
            <p>
              <button
                onClick={() =>
                  deleteMessage({
                    variables: {
                      id: node.id
                    }
                  })
                }
              >
                &times; Delete
              </button>
            </p>
          </li>
        ))}
      </ul>
    </>
  )
}

export default App
