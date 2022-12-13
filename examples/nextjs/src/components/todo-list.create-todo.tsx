import { TodoCreateDocument } from 'graphql/schema'
import { useMutation } from 'urql'
import { ChangeEvent, FormEvent, useMemo, useState } from 'react'

const TodoListCreateTodo = ({ todoListId }: { todoListId: string }) => {
  const context = useMemo(() => ({ additionalTypenames: ['Todo'] }), [])

  const [title, setTitle] = useState<string>('')

  const [{ fetching }, createTodo] = useMutation(TodoCreateDocument)

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    createTodo({ title, todoListId }, context)
    setTitle('')
  }

  const onChangeTitle = (event: ChangeEvent<HTMLInputElement>) =>
    setTitle(event.target.value)

  return (
    <form
      className="flex items-center p-3 space-x-2 border-2 border-gray-200 border-dashed rounded-lg dark:border-gray-800"
      onSubmit={onSubmit}
    >
      <input
        required
        value={title}
        placeholder="Todo title"
        onChange={onChangeTitle}
        className="block w-full px-2 py-1 text-sm text-gray-900 placeholder-gray-400 border border-gray-300 rounded-md bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
      />
      <button
        disabled={fetching}
        className="px-2 py-1 text-sm text-white bg-blue-800 rounded-md whitespace-nowrap disabled:bg-blue-400"
      >
        {fetching ? 'Adding...' : 'Add Todo'}
      </button>
    </form>
  )
}

export default TodoListCreateTodo
