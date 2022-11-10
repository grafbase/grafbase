import Img from 'components/img'
import { gql, useApolloClient, useMutation } from '@apollo/client'
import formatDistanceToNow from 'date-fns/formatDistanceToNow'
import { ItemOneQuery } from 'gql/graphql'
import useViewer from 'hooks/use-viewer'

const ITEM_COMMENT_DELETE_MUTATION = gql`
  mutation ItemCommentDelete($id: ID!) {
    commentDelete(by: { id: $id }) {
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
    <div className="w-full border">
      <div className="items-center justify-between pr-4 sm:flex bg-gray-50">
        <div className="flex items-center space-x-4">
          <Img src={imageUrl} alt={name} className="w-10 h-10" />
          <span className="text-base">{name}</span>
        </div>
        <div className="flex items-center p-2 space-x-4 sm:p-0">
          {isSignedInUserComment && (
            <div className="text-sm border whitespace-nowrap">
              <button
                onClick={onDelete}
                className="px-2 text-gray-700 hover:bg-gray-200"
              >
                Delete
              </button>
            </div>
          )}
          <time className="text-xs text-gray-600">
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
