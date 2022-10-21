import { gql, useQuery } from '@apollo/client'
import Head from 'components/head'
import ItemList from 'components/item-list'
import { ItemsListQuery } from 'gql/graphql'
import type { NextPage } from 'next'
import Link from 'next/link'
import { useAuth } from '@clerk/nextjs'
import { graphQlRequestClient } from 'lib/request'

const ITEMS_LIST_QUERY = gql`
  query ItemsList($after: String) {
    itemCollection(first: 4, after: $after) {
      pageInfo {
        endCursor
        hasNextPage
      }
      edges {
        node {
          id
          title
          comments(first: 100) {
            edges {
              __typename
            }
          }
          votes(first: 100) {
            edges {
              node {
                id
                positive
                user {
                  id
                }
              }
            }
          }
          author {
            id
            name
            imageUrl
          }
          url
          createdAt
        }
      }
    }
  }
`

const Home: NextPage = (props: { data: ItemsListQuery }) => {
  const { isSignedIn } = useAuth()
  const {
    data: clientData,
    loading,
    error,
    fetchMore
  } = useQuery<ItemsListQuery>(ITEMS_LIST_QUERY, {
    skip: !isSignedIn,
    notifyOnNetworkStatusChange: true
  })

  const data = clientData ?? props.data

  return (
    <>
      <Head>
        <title>Grafnews | Feed</title>
      </Head>
      <div className="space-y-8">
        <div className="bg-indigo-600 p-4 border border-b-4 border-indigo-300 space-y-4 sm:space-y-0 sm:flex items-center justify-between sm:space-x-4">
          <h2 className="text-white text-2xl ">
            Grafbase is open to everyone, start building your frontend with the
            next-gen GraphQL platform!
          </h2>
          <div>
            <a
              href="https://grafbase.com/register"
              className="border border-white px-3 py-2 text-xl text-white whitespace-nowrap"
            >
              Try it
            </a>
          </div>
        </div>
        {(loading || !!error) && !data?.itemCollection?.edges?.length && (
          <>
            <div className="animate-pulse bg-gray-200 p-4 border border-gray-400 h-24 border-b-4 w-full" />
            <div className="animate-pulse bg-gray-200 p-4 border border-gray-400 h-24 border-b-4 w-full" />
            <div className="animate-pulse bg-gray-200 p-4 border border-gray-400 h-24 border-b-4 w-full" />
          </>
        )}
        {!!error && (
          <div className="bg-red-500 min-h-24 w-full flex flex-col space-y-6 items-center justify-center py-6">
            <div className="text-lg text-white">
              Something went wrong in the API.
            </div>
          </div>
        )}
        {!loading && !error && !data?.itemCollection?.edges?.length && (
          <div className="border border-black bg-gray-200 min-h-24 w-full flex flex-col space-y-6 items-center justify-center py-6">
            <div className="text-lg">No items yet.</div>
            <Link href="/item/submit" passHref>
              <a>
                <button className="px-2 py-1 bg-black text-white hover:bg-gray-700">
                  Submit item
                </button>
              </a>
            </Link>
          </div>
        )}
        {data?.itemCollection?.edges?.map(
          (edge) => !!edge && <ItemList key={edge.node.id} {...edge.node} />
        )}
        {!!data?.itemCollection?.pageInfo?.hasNextPage && isSignedIn && (
          <div className="text-center">
            <button
              onClick={() =>
                fetchMore({
                  variables: {
                    after: data?.itemCollection?.pageInfo?.endCursor
                  }
                })
              }
              className="border border-gray-300 text-lg w-fu px-2 py-1 font-semibold text-gray-700 hover:bg-gray-50"
            >
              Load More {loading ? '...' : ''}
            </button>
          </div>
        )}
        {!!data?.itemCollection?.pageInfo?.hasNextPage && !isSignedIn && (
          <div className="text-center">
            <Link href="/login" passHref>
              <a className="border border-gray-300 text-lg w-fu px-2 py-1 font-semibold text-gray-700 hover:bg-gray-50">
                Sign In to load More
              </a>
            </Link>
          </div>
        )}
      </div>
    </>
  )
}

export const getStaticProps = async () => {
  const data = await graphQlRequestClient.request(ITEMS_LIST_QUERY)

  return {
    props: {
      data: data ?? null
    }
  }
}

export default Home
