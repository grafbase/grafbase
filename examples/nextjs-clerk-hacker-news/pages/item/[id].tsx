import { gql, useMutation, useQuery } from '@apollo/client'
import { SignedIn, useAuth } from '@clerk/nextjs'
import Img from 'components/img'
import ItemAddComment from 'components/item-add-comment'
import ItemComment from 'components/item-comment'
import ItemVotes from 'components/item-votes'
import formatDistanceToNow from 'date-fns/formatDistanceToNow'
import { ItemOneQuery } from 'gql/graphql'
import useViewer from 'hooks/use-viewer'
import Head from 'next/head'
import { useRouter } from 'next/router'
import { graphQlRequestClient } from 'lib/request'
import { GetServerSideProps } from 'next'
import Link from 'next/link'

const ITEM_QUERY = gql`
  query ItemOne($id: ID!, $afterComments: String) {
    item(by: { id: $id }) {
      id
      title
      comments(first: 6, after: $afterComments) {
        pageInfo {
          endCursor
          hasNextPage
        }
        edges {
          node {
            id
            content
            createdAt
            author {
              id
              name
              imageUrl
            }
          }
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
`

const ITEM_DELETE_MUTATION = gql`
  mutation ItemOneDelete($id: ID!) {
    itemDelete(id: $id) {
      deletedId
    }
  }
`

const ItemIdPage = (props: { data: ItemOneQuery }) => {
  const { isSignedIn } = useAuth()
  const { query, replace } = useRouter()
  const { viewer } = useViewer()
  const {
    data: clientData,
    loading,
    error,
    fetchMore
  } = useQuery<ItemOneQuery>(ITEM_QUERY, {
    skip: !isSignedIn,
    notifyOnNetworkStatusChange: true,
    variables: { id: query.id }
  })

  const data = clientData ?? props.data

  const [deleteMutation] = useMutation(ITEM_DELETE_MUTATION)

  if (loading && !data?.item) {
    return (
      <div className="flex">
        <div className="animate-pulse bg-gray-200 h-[136.5px] w-[32px]" />
        <div className="ml-4 animate-pulse bg-gray-200 h-[39px] w-[250px]" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="bg-red-500 min-h-24 w-full flex flex-col space-y-6 items-center justify-center py-6">
        <div className="text-lg text-white">
          Something went wrong in the API.
        </div>
      </div>
    )
  }

  const { id, title, comments, createdAt, url, votes, author } = data?.item!

  const isSessionUserItem = author.id === viewer?.id

  const onDelete = () => {
    if (confirm('Are you sure you want to delete this item?')) {
      deleteMutation({ variables: { id } }).then(() => replace('/'))
    }
  }

  return (
    <div>
      <Head>
        <title>{title} | Grafnews</title>
      </Head>
      <div className="flex">
        <div className="flex flex-col border border-black">
          <ItemVotes itemId={id} votes={votes} />
        </div>
        <div className="pl-4 flex-1">
          <h1 className="text-5xl font-bold">{title}</h1>
          <div className="mt-4">
            <div className="bg-gray-100 p-4 text-xl  text-gray-800">
              <a href={url} target="_blank" rel="noreferrer">
                {url}
              </a>
            </div>
          </div>
          <div className="flex justify-end space-x-2 items-center mt-4">
            {isSessionUserItem && (
              <button
                onClick={onDelete}
                className="text-gray-700 hover:bg-red-200 px-2 border"
              >
                Delete
              </button>
            )}
            <span className="text-gray-500">
              <time className="font-semibold text-gray-700">
                {!!createdAt &&
                  formatDistanceToNow(Date.parse(createdAt), {
                    addSuffix: true
                  })}
              </time>{' '}
              by {author.name}
            </span>
            <Img src={author.imageUrl} alt={author.name} className="h-7 w-7" />
          </div>
        </div>
      </div>
      <hr className="mt-6" />
      <div>
        <SignedIn>
          <ItemAddComment itemId={id} />
        </SignedIn>
        <div>
          <h2 className="mt-6 text-lg mb-5">
            {comments?.edges?.length
              ? `Comments (${comments?.edges?.length})`
              : 'No comments yet'}
          </h2>
          <div className="space-y-4">
            {comments?.edges?.map((edge) => {
              if (!edge?.node) {
                return null
              }

              return <ItemComment key={edge.node.id} {...edge.node} />
            })}
            {!!data?.item?.comments?.pageInfo?.hasNextPage && isSignedIn && (
              <div className="text-center">
                <button
                  onClick={() =>
                    fetchMore({
                      variables: {
                        afterComments: data?.item?.comments?.pageInfo?.endCursor
                      }
                    })
                  }
                  className="border border-gray-300 text-lg w-fu px-2 py-1 font-semibold text-gray-700 hover:bg-gray-50"
                >
                  Load More {loading ? '...' : ''}
                </button>
              </div>
            )}
            {!!data?.item?.comments?.pageInfo?.hasNextPage && !isSignedIn && (
              <div className="text-center">
                <Link href="/login" passHref>
                  <a className="border border-gray-300 text-lg w-fu px-2 py-1 font-semibold text-gray-700 hover:bg-gray-50">
                    Sign In to load More
                  </a>
                </Link>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export const getServerSideProps: GetServerSideProps = async ({ params }) => {
  const data = await graphQlRequestClient.request(ITEM_QUERY, {
    id: params?.id
  })

  return {
    props: {
      data: data ?? null
    }
  }
}

export default ItemIdPage
