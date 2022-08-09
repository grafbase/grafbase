import {
  TodoDeleteDocument,
  TodoFragment,
  TodoUpdateDocument
} from 'graphql/schema'
import { useMutation } from 'urql'
import { useMemo, useState } from 'react'
import { TrashIcon } from '@heroicons/react/outline'
import Spinner from 'components/spinner'
import debounce from 'lodash.debounce'

const TodoListTodo = (props: {
  title: string
  id: string
  complete?: boolean | null
}) => {
  const { id, title, complete } = props
  const contextDeleteTodoList = useMemo(
    () => ({ additionalTypenames: ['TodoList'] }),
    []
  )
  const [{ fetching }, todoDelete] = useMutation(TodoDeleteDocument)
  const [{}, todoUpdate] = useMutation(TodoUpdateDocument)

  const [completed, setCompleted] = useState(!!complete)

  const onTodoUpdate = (todoProps: Partial<TodoFragment>) =>
    todoUpdate({ ...props, ...todoProps })

  const onTitleChange = debounce((title: string) => {
    onTodoUpdate({ title })
  }, 500)

  const onCheckboxClick = () =>
    setCompleted((c) => {
      onTodoUpdate({ complete: !c })
      return !c
    })

  return (
    <div
      className={`relative rounded-md border p-3 overflow-hidden ${
        completed
          ? 'bg-emerald-800 border-emerald-600'
          : 'bg-zinc-50 dark:bg-gray-700 border-gray-200 dark:border-transparent'
      }`}
    >
      {completed && (
        <div className="absolute left-0 font-bold tracking-wider text-white text-8xl -top-3 text-opacity-5">
          DONE
        </div>
      )}
      <div className="relative">
        <div className="flex justify-between gap-4">
          <div className="flex space-x-1.5 items-center truncate" title={title}>
            <input
              type="checkbox"
              defaultChecked={completed}
              className="text-green-600 bg-white border-gray-200 rounded dark:border-gray-500 dark:bg-black accent-green-600 hover:bg-green-600 focus:ring-0"
              onClick={onCheckboxClick}
            />
            <input
              defaultValue={title}
              className={`bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400 ${
                completed ? 'text-white' : 'text-black dark:text-white'
              }`}
              onChange={(e) => onTitleChange(e?.target?.value)}
            />
          </div>
          <button
            className="text-gray-400 transition hover:text-red-400"
            onClick={() => todoDelete({ id }, contextDeleteTodoList)}
          >
            {fetching ? <Spinner /> : <TrashIcon className="w-4 h-4" />}
          </button>
        </div>
        <div className="flex justify-between mt-2 text-sm">
          <div
            className={`text-xs px-1 py-0.5 rounded ${
              completed
                ? 'bg-green-600 text-white'
                : 'bg-gray-300 dark:bg-gray-600 text-black dark:text-white'
            }`}
          >
            {completed ? 'Completed' : 'Not completed'}
          </div>
        </div>
      </div>
    </div>
  )
}

export default TodoListTodo
