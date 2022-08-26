import { createSignal } from 'solid-js'
import { createMutation } from 'solid-urql'

import { TodoListCreateDocument } from '~/graphql/schema'

const TodoListCreateContext = { additionalTypenames: ['TodoList'] }

const TodoListCreate = () => {
  const [title, setTitle] = createSignal('')

  const [creating, createTodoList] = createMutation(TodoListCreateDocument)

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        createTodoList({ title: title() }, TodoListCreateContext).then(() =>
          setTitle('')
        )
      }}
      class="h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]"
    >
      <h2 class="text-xl font-bold text-gray-900 dark:text-gray-300">
        New List
      </h2>
      <fieldset disabled={creating().fetching} class="flex space-x-2">
        <input
          required
          type="text"
          placeholder="Todo list title"
          value={title()}
          onInput={(e) => setTitle(e.currentTarget.value)}
          class="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          type="submit"
          disabled={creating().fetching}
          class="px-5 py-1 text-sm text-white bg-purple-600 rounded-md disabled:bg-purple-500 min-w-[110px]"
        >
          {creating().fetching ? 'Creating...' : 'Create'}
        </button>
      </fieldset>
    </form>
  )
}

export default TodoListCreate
