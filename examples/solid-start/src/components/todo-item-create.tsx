import { createServerAction$, redirect } from 'solid-start/server'
import { Mutation, TodoCreateDocument } from '~/graphql/schema'
import { grafbase } from '~/utils/grafbase'

const TodoItemCreate = ({ todoListId }: { todoListId: string }) => {
  const [creatingTodo, createTodo] = createServerAction$(
    async (form: FormData) => {
      const vars = Object.fromEntries(form)
      await grafbase.request<Mutation>(TodoCreateDocument, vars)
      return redirect('/')
    }
  )

  let inputRef: HTMLInputElement

  return (
    <createTodo.Form
      onSubmit={(e) => {
        if (!inputRef.value.trim()) e.preventDefault()
        setTimeout(() => (inputRef.value = ''))
      }}
      class="p-3 border-2 border-gray-200 border-dashed rounded-lg dark:border-gray-800"
    >
      <fieldset disabled={creatingTodo.pending} class="flex space-x-2">
        <input type="hidden" name="todoListId" value={todoListId} hidden />
        <input
          ref={inputRef}
          required
          type="text"
          name="title"
          placeholder="Todo title"
          class="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-md bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          type="submit"
          disabled={creatingTodo.pending}
          class="px-2 py-1 text-sm text-white bg-blue-800 rounded-md whitespace-nowrap disabled:bg-blue-400 min-w-[80px]"
        >
          {creatingTodo.pending ? 'Adding...' : 'Add Todo'}
        </button>
      </fieldset>
    </createTodo.Form>
  )
}

export default TodoItemCreate
