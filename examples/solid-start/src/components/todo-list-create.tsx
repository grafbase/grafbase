import { createServerAction$, redirect } from 'solid-start/server'
import { Mutation, TodoListCreateDocument } from '~/graphql/schema'
import { grafbase } from '~/utils/grafbase'

const TodoListCreate = () => {
  const [{ pending }, { Form }] = createServerAction$(async (form: FormData) => {
    const vars = Object.fromEntries(form)
    await grafbase.request<Mutation>(TodoListCreateDocument, vars)
    return redirect('/')
  })

  let inputRef: HTMLInputElement

  return (
    <Form
      onSubmit={(e) => {
        if (!inputRef.value.trim()) e.preventDefault()
        setTimeout(() => (inputRef.value = ''))
      }}
      class='h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]'
    >
      <h2 class='text-xl font-bold text-gray-900 dark:text-gray-300'>New List</h2>
      <fieldset disabled={pending} class='flex space-x-2'>
        <input
          ref={inputRef}
          required
          type='text'
          name='title'
          placeholder='Todo list title'
          class='block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500'
        />
        <button
          type='submit'
          disabled={pending}
          class='px-5 py-1 text-sm text-white bg-purple-600 rounded-md disabled:bg-purple-500 min-w-[110px]'
        >
          {pending ? 'Creating...' : 'Create'}
        </button>
      </fieldset>
    </Form>
  )
}

export default TodoListCreate
