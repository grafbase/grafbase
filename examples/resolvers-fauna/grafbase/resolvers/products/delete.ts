import { Client, fql, FaunaError } from 'fauna'
import { GraphQLError } from 'graphql'

const client = new Client()

export default async function ProductsDelete(_, { by }) {
  const { id } = by

  try {
    const documentQuery = fql`
      products.byId(${id}).delete()
    `

    await client.query(documentQuery)

    return { deleted: true }
  } catch (error) {
    if (error instanceof FaunaError) {
      throw new GraphQLError(error?.message)
    }

    return { deleted: false }
  }
}
