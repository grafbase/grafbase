import { TodoListCreateDocument } from 'graphql/schema'
import { useMutation } from 'urql'
import { FormEvent, useMemo, useState } from 'react'

const NewTodoList = () => {
  const context = useMemo(() => ({ additionalTypenames: ['TodoList'] }), [])
  const [title, setTitle] = useState<string>('')

  const [{ fetching }, createTodoList] = useMutation(TodoListCreateDocument)

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    createTodoList({ title }, context)
    setTitle('')
  }

  return (
    <form
      className="h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]"
      onSubmit={onSubmit}
    >
      <h2 className="text-xl font-bold text-gray-900 dark:text-gray-300">
        New List
      </h2>
      <div className="flex space-x-3">
        <input
          required
          value={title}
          placeholder="Todo list title"
          onChange={(e) => setTitle(e.target.value)}
          className="block w-full px-2 py-1 text-sm text-gray-900 placeholder-gray-400 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          disabled={fetching}
          className="px-2 py-1 text-sm text-white bg-purple-600 rounded-md disabled:bg-purple-500"
        >
          {fetching ? 'Creating...' : 'Create'}
        </button>
      </div>
    </form>
  )
}

export default NewTodoList
