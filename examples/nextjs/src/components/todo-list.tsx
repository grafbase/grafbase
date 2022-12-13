import {
  TodoListFragment,
  TodoListDeleteDocument,
  TodoListUpdateDocument
} from 'graphql/schema'
import { useMutation } from 'urql'
import { useMemo, useRef } from 'react'
import TodoListCreateTodo from 'components/todo-list.create-todo'
import TodoListTodo from 'components/todo-list.todo'
import {
  DotsVerticalIcon,
  PencilIcon,
  TrashIcon
} from '@heroicons/react/outline'
import getColor from 'utils/get-color'
import Spinner from 'components/spinner'
import Dropdown from 'components/dropdown'
import debounce from 'lodash.debounce'

const TodoList = (props: TodoListFragment) => {
  const inputRef = useRef<HTMLInputElement>(null)
  const { id, title, todos } = props
  const contextDeleteTodoList = useMemo(
    () => ({ additionalTypenames: ['TodoList', 'Todo'] }),
    []
  )
  const [{ fetching }, todoListDelete] = useMutation(TodoListDeleteDocument)
  const [{}, todoListUpdate] = useMutation(TodoListUpdateDocument)

  const dropdownOptions = useMemo(
    () => [
      {
        name: 'Rename',
        icon: PencilIcon,
        onClick: () => setTimeout(() => inputRef.current?.focus(), 100)
      },
      {
        name: 'Delete',
        icon: TrashIcon,
        onClick: () => todoListDelete({ id }, contextDeleteTodoList)
      }
    ],
    [contextDeleteTodoList, id, todoListDelete]
  )

  const onTitleChange = debounce((title: string) => {
    todoListUpdate({ id, title })
  }, 500)

  return (
    <div className="space-y-4 flex-1 min-w-[300px]">
      <div
        className="flex justify-between border-b-2 "
        title={title || ''}
        style={{ borderColor: getColor(id) }}
      >
        <h2 className="text-xl font-bold truncate">
          <input
            ref={inputRef}
            defaultValue={title || ''}
            className="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
            onChange={(e) => onTitleChange(e.target.value)}
          />
        </h2>
        <div className="relative z-20">
          {fetching ? (
            <Spinner />
          ) : (
            <Dropdown options={dropdownOptions}>
              <DotsVerticalIcon className="w-5 h-5 text-gray-400 transition hover:text-red-400" />
            </Dropdown>
          )}
        </div>
      </div>
      <div className="space-y-4">
        {todos?.edges?.map(
          (edge) => edge?.node && <TodoListTodo key={edge.node.id} {...edge.node} />
        )}
      </div>
      <TodoListCreateTodo todoListId={id} />
    </div>
  )
}

export default TodoList
