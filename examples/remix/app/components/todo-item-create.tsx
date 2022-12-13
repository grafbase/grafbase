import { useFetcher } from '@remix-run/react'
import { useEffect, useRef } from 'react'

const TodoItemCreate = ({ todoListId }: { todoListId: string }) => {
  const { Form, submission } = useFetcher()
  const isCreating = submission?.formData.get('_action') === 'todo-item-create'

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
      className="p-3 border-2 border-gray-200 border-dashed rounded-lg dark:border-gray-800"
    >
      <fieldset disabled={isCreating} className="flex space-x-2">
        <input type="hidden" name="todoListId" value={todoListId} hidden />
        <input
          required
          type="text"
          name="title"
          placeholder="Todo title"
          className="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-md bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          type="submit"
          name="_action"
          value="todo-item-create"
          disabled={isCreating}
          className="px-2 py-1 text-sm text-white bg-blue-800 rounded-md whitespace-nowrap disabled:bg-blue-400 min-w-[80px]"
        >
          {isCreating ? 'Adding...' : 'Add Todo'}
        </button>
      </fieldset>
    </Form>
  )
}

export default TodoItemCreate
