import { createSignal } from 'solid-js'
import { createMutation } from 'solid-urql'

import { TodoCreateDocument } from '~/graphql/schema'

const TodoCreateContext = { additionalTypenames: ['Todo'] }

const TodoItemCreate = ({ todoListId }: { todoListId: string }) => {
  const [title, setTitle] = createSignal('')

  const [creating, createTodo] = createMutation(TodoCreateDocument)

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        createTodo({ title: title(), todoListId }, TodoCreateContext).then(() =>
          setTitle('')
        )
      }}
      class="p-3 border-2 border-gray-200 border-dashed rounded-lg dark:border-gray-800"
    >
      <fieldset disabled={creating().fetching} class="flex space-x-2">
        <input
          required
          type="text"
          placeholder="Todo title"
          value={title()}
          onInput={(e) => setTitle(e.currentTarget.value)}
          class="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-md bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          type="submit"
          disabled={creating().fetching}
          class="px-2 py-1 text-sm text-white bg-blue-800 rounded-md whitespace-nowrap disabled:bg-blue-400 min-w-[80px]"
        >
          {creating().fetching ? 'Adding...' : 'Add Todo'}
        </button>
      </fieldset>
    </form>
  )
}

export default TodoItemCreate
