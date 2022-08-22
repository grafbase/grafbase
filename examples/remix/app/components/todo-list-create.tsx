import { useFetcher } from '@remix-run/react'
import { useEffect, useRef } from 'react'

const TodoListCreate = () => {
  const { Form, submission } = useFetcher()
  const isCreating = submission?.formData.get('_action') === 'todo-list-create'

  const formRef = useRef<HTMLFormElement>(null)

  useEffect(() => {
    if (!isCreating) {
      formRef.current?.reset()
    }
  }, [isCreating])

  return (
    <Form
      ref={formRef}
      method="post"
      className="h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]"
    >
      <h2 className="text-xl font-bold text-gray-900 dark:text-gray-300">
        New List
      </h2>
      <fieldset disabled={isCreating} className="flex space-x-2">
        <input
          required
          type="text"
          name="title"
          placeholder="Todo list title"
          className="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          type="submit"
          name="_action"
          value="todo-list-create"
          disabled={isCreating}
          className="px-5 py-1 text-sm text-white bg-purple-600 rounded-md disabled:bg-purple-500 min-w-[110px]"
        >
          {isCreating ? 'Creating...' : 'Create'}
        </button>
      </fieldset>
    </Form>
  )
}

export default TodoListCreate
