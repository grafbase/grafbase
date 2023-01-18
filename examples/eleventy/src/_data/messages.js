const EleventyFetch = require('@11ty/eleventy-fetch')

const url = process.env.GRAFBASE_API_URL || 'http://localhost:4000/graphql'

const GetAllMessagesQuery = /* GraphQL */ `
  query GetAllMessagesQuery($first: Int, after: String) {
    messageCollection(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          body
          author
          createdAt
        }
      }
    }
  }
`

module.exports = async function () {
  // Re-use this to loop through pages of 100 using the after cursor
  const { data } = await EleventyFetch(url, {
    duration: '2s',
    type: 'json',
    fetchOptions: {
      headers: {
        'content-type': 'application/json',
        'x-api-key': process.env.GRAFBASE_API_KEY
      },
      method: 'POST',
      body: JSON.stringify({
        query: GetAllMessagesQuery,
        variables: {
          first: 100,
          after: null
        }
      })
    }
  })

  return data?.messageCollection?.edges || []
}
