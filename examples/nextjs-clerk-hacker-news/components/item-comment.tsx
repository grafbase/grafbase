import Img from 'components/img'
import { gql, useApolloClient, useMutation } from '@apollo/client'
import formatDistanceToNow from 'date-fns/formatDistanceToNow'
import { ItemOneQuery } from 'gql/graphql'
import useViewer from 'hooks/use-viewer'

const ITEM_COMMENT_DELETE_MUTATION = gql`
  mutation ItemCommentDelete($id: ID!) {
    commentDelete(id: $id) {
      deletedId
    }
  }
`

const ItemComment = (
  props: NonNullable<
    NonNullable<
      NonNullable<NonNullable<ItemOneQuery['item']>['comments']>['edges']
    >[0]
  >['node']
) => {
  const { viewer } = useViewer()
  const client = useApolloClient()
  const {
    id,
    content,
    createdAt,
    author: { id: authorId, name, imageUrl }
  } = props

  const [deleteMutation] = useMutation(ITEM_COMMENT_DELETE_MUTATION)

  const isSignedInUserComment = authorId === viewer?.id

  const onDelete = () => {
    return alert('Not working yet')

    if (confirm('Are you sure you want to delete this comment?')) {
      deleteMutation({ variables: { id } }).then(() =>
        client.refetchQueries({
          include: ['ItemOne']
        })
      )
    }
  }

  return (
    <div className="border w-full">
      <div className="sm:flex justify-between items-center bg-gray-50 pr-4">
        <div className="flex items-center space-x-4">
          <Img src={imageUrl} alt={name} className="h-10 w-10" />
          <span className="text-base">{name}</span>
        </div>
        <div className="p-2 sm:p-0 flex items-center space-x-4">
          {isSignedInUserComment && (
            <div className="text-sm border whitespace-nowrap">
              <button
                onClick={onDelete}
                className="text-gray-700 hover:bg-gray-200 px-2"
              >
                Delete
              </button>
            </div>
          )}
          <time className="text-gray-600 text-xs">
            {!!createdAt &&
              formatDistanceToNow(Date.parse(createdAt), { addSuffix: true })}
          </time>
        </div>
      </div>
      <div className="p-4 text-gray-700">{content}</div>
    </div>
  )
}

export default ItemComment
