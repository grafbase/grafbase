import NextAuth, { NextAuthOptions } from 'next-auth'
import GitHubProvider from 'next-auth/providers/github'
import jsonwebtoken from 'jsonwebtoken'
import { JWT } from 'next-auth/jwt'

export const authOptions: NextAuthOptions = {
  debug: true,
  providers: [
    GitHubProvider({
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!
    })
  ],
  jwt: {
    encode: ({ secret, token }) => {
      const encodedToken = jsonwebtoken.sign(
        {
          ...token,
          iss: process.env.GRAFBASE_ISSUER_URL,
          exp: Math.floor(Date.now() / 1000) + 60 * 60
        },
        secret
      )
      return encodedToken
    },
    decode: async ({ secret, token }) => {
      const decodedToken = jsonwebtoken.verify(token!, secret)
      return decodedToken as JWT
    }
  }
  // Use this with `account` to get role/group
  // if the user is stored in Grafbase
  // callbacks: {
  //   async jwt({ token }) {
  // // Fetch user from backend to get role
  //     return {
  //       ...token,
  //       groups: ['admin']
  //     }
  //   }
  // }
}

export default NextAuth(authOptions)
