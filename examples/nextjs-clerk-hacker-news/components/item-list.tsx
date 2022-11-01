import Img from 'components/img'
import ItemVotes from 'components/item-votes'
import formatDistanceToNow from 'date-fns/formatDistanceToNow'
import { ItemsListQuery } from 'gql/graphql'
import Link from 'next/link'

const ItemList = (
  props: NonNullable<
    NonNullable<NonNullable<ItemsListQuery['itemCollection']>['edges']>[0]
  >['node']
) => {
  const { id, url, title, votes, createdAt, comments, author } = props

  return (
    <div className="border w-full border-b-4 border-gray-500">
      <div className="flex">
        <div className="flex flex-col border-r border-black">
          <ItemVotes itemId={id} votes={votes} />
        </div>
        <div className="flex flex-col space-y-4 pt-4 w-full">
          <a
            href={url}
            target="_blank"
            rel="noreferrer"
            className="text-2xl font-semibold hover:text-indigo-700"
          >
            <div className="px-4">
              {title}
              <div className="text-gray-500 mt-1 text-sm">{url}</div>
            </div>
          </a>
          <div className="sm:flex justify-between items-center bg-gray-200 w-full flex-1 px-4 py-2">
            <Link
              href={{
                pathname: '/item/[id]',
                query: {
                  id
                }
              }}
              passHref
              className="text-lg  text-gray-700"
            >
              {`${comments?.edges?.length} ${
                comments?.edges?.length === 1 ? 'comment' : 'comments'
              } `}
            </Link>
            <div className="flex space-x-2 items-center">
              <span className="text-gray-700">
                <time className="font-semibold text-gray-800">
                  {!!createdAt &&
                    formatDistanceToNow(Date.parse(createdAt), {
                      addSuffix: true
                    })}
                </time>{' '}
                by {author.name}
              </span>
              <Img
                src={author.imageUrl}
                alt={author.name}
                className="h-7 w-7"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ItemList
