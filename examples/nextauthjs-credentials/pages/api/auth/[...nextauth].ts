import NextAuth from 'next-auth'
import Credentials from 'next-auth/providers/credentials'
import { gql, GraphQLClient } from 'graphql-request'
import { compare, hash } from 'bcrypt'

const grafbase = new GraphQLClient(process.env.GRAFBASE_API_URL as string, {
  headers: {
    'x-api-key': process.env.GRAFBASE_API_KEY as string
  }
})

const GetUserByUsername = gql`
  query GetUserByUsername($username: String!) {
    user(by: { username: $username }) {
      id
      password
    }
  }
`

const CreateUserByUsername = gql`
  mutation CreateUserByUsername($username: String!, $password: String!) {
    userCreate(input: { username: $username, password: $password }) {
      user {
        id
        username
      }
    }
  }
`

export const authOptions = {
  providers: [
    Credentials({
      name: 'Credentials',
      credentials: {
        username: {
          label: 'Username',
          type: 'text',
          placeholder: 'grafbase'
        },
        password: { label: 'Password', type: 'password' }
      },
      async authorize(credentials) {
        const { username, password } = credentials as {
          username: string
          password: string
        }

        const { user } = await grafbase.request(GetUserByUsername, {
          username
        })

        if (!user) {
          const { userCreate } = await grafbase.request(CreateUserByUsername, {
            username,
            password: await hash(password, 12)
          })

          return {
            id: userCreate.id,
            username
          }
        }

        const isValid = await compare(password, user.password)

        if (!isValid) {
          throw new Error('Wrong credentials. Try again.')
        }

        return user
      }
    })
  ]
}

export default NextAuth(authOptions)
