import { gql, useQuery } from '@apollo/client'
import Img from 'components/img'
import formatDistanceToNow from 'date-fns/formatDistanceToNow'
import { UsersListQuery } from 'gql/graphql'
import Head from 'next/head'
import Link from 'next/link'

const USERS_LIST_QUERY = gql`
  query UsersList($after: String) {
    userCollection(first: 10, after: $after) {
      pageInfo {
        endCursor
        hasNextPage
      }
      edges {
        node {
          id
          name
          imageUrl
          createdAt
        }
      }
    }
  }
`

const UsersPage = () => {
  const { data, loading, error, fetchMore } =
    useQuery<UsersListQuery>(USERS_LIST_QUERY)

  return (
    <div>
      <Head>
        <title>Users | Grafnews</title>
      </Head>
      <h1 className="text-5xl font-bold">Users</h1>
      <div className="border-b-4 mt-6 max-w-sm border-black" />
      <p className="text-xl mt-4 text-gray-600">
        All the users that have joined Grafnews
      </p>
      <h3 className="mt-8 text-2xl font-semibold">
        Total ({data?.userCollection?.edges?.length || 0})
      </h3>
      <div className="space-y-4 mt-6">
        {(loading || !!error) && !data?.userCollection?.edges?.length && (
          <>
            <div className="animate-pulse bg-gray-200 p-4 border h-11 border-b-4 w-full" />
            <div className="animate-pulse bg-gray-200 p-4 border h-11 border-b-4 w-full" />
            <div className="animate-pulse bg-gray-200 p-4 border h-11 border-b-4 w-full" />
          </>
        )}
        {!loading && !error && !data?.userCollection?.edges?.length && (
          <div className="border border-black bg-gray-200 min-h-24 w-full flex flex-col space-y-6 items-center justify-center py-6">
            <div className="text-lg">No users yet.</div>
            <Link href="//login" passHref>
              <a>
                <button className="px-2 py-1 bg-black text-white hover:bg-gray-700">
                  Be the first one
                </button>
              </a>
            </Link>
          </div>
        )}
        {data?.userCollection?.edges?.map((edge) => {
          if (!edge?.node) {
            return null
          }

          const { id, name, imageUrl, createdAt } = edge.node

          return (
            <div
              key={id}
              className="border border-b-4 border-gray-300 flex items-center justify-between pr-4"
            >
              <div className="flex items-center space-x-4">
                <Img src={imageUrl} className="h-12 w-12" alt={name} />
                <span className="text-xl">{name}</span>
              </div>
              <div className="hidden sm:block">
                <time className="text-gray-600">
                  Joined{' '}
                  {!!createdAt &&
                    formatDistanceToNow(Date.parse(createdAt), {
                      addSuffix: true
                    })}
                </time>
              </div>
            </div>
          )
        })}
        {!!data?.userCollection?.pageInfo?.hasNextPage && (
          <div className="text-center">
            <button
              onClick={() =>
                fetchMore({
                  variables: {
                    after: data?.userCollection?.pageInfo?.endCursor
                  }
                })
              }
              className="border border-gray-300 text-lg w-fu px-2 py-1 font-semibold text-gray-700 hover:bg-gray-50"
            >
              Load More {loading ? '...' : ''}
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export default UsersPage
