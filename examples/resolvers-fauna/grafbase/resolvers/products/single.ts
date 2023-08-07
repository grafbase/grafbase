import { Client, fql, FaunaError } from 'fauna'
import { GraphQLError } from 'graphql'

const client = new Client()

export default async function ProductsSingle(_, { by }) {
  const { id } = by

  if (Object.entries(by).length === 0) {
    throw new GraphQLError('You must provide at least one field to fetch by.')
  }

  try {
    const documentQuery = fql`
      products.byId(${id}) {
        id,
        name,
        price
      }
    `

    const { data } = await client.query(documentQuery)

    return data
  } catch (error) {
    if (error instanceof FaunaError) {
      console.log(error)
    }

    return null
  }
}
