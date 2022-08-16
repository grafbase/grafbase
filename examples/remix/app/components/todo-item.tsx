import { useFetcher } from '@remix-run/react'
import debounce from 'lodash.debounce'
import { TrashIcon } from '~/components/icons'

type Props = {
  id: string
  title: string
  complete: boolean
}

const TodoItem = (props: Props) => {
  const { id, title, complete } = props
  const { Form, submission, submit } = useFetcher()
  const completed = submission
    ? Boolean(submission.formData.get('complete'))
    : complete

  const isDeleting = submission?.formData.get('_action') === 'todo-item-delete'

  const onTodoUpdate = (data: Partial<Props>) => {
    const form = new FormData()
    form.set('_action', 'todo-item-update')
    Object.entries({ ...props, ...data }).forEach(([key, value]) =>
      form.set(key, `${value}`)
    )
    return submit(form, { method: 'post' })
  }
  const onTitleChange = debounce(
    (title: string) => onTodoUpdate({ title }),
    500
  )
  const onChecked = (c: boolean) => onTodoUpdate({ complete: c })

  return (
    <div
      hidden={isDeleting}
      className={`relative p-3 overflow-hidden border rounded-md ${
        completed
          ? 'bg-emerald-200 dark:bg-emerald-800 border-emerald-600'
          : 'bg-zinc-50 dark:bg-gray-700 border-gray-200 dark:border-transparent'
      }`}
    >
      {completed && (
        <div className="absolute inset-y-0 -inset-x-2.5 font-bold leading-[70px] text-black text-[120px] text-opacity-5 uppercase">
          done
        </div>
      )}
      <div className="relative">
        <div className="flex justify-between gap-4">
          <fieldset
            title={title}
            className="flex space-x-1.5 items-center truncate"
          >
            <input
              type="checkbox"
              name="complete"
              checked={completed}
              onChange={(e) => onChecked(e.target.checked)}
              className="text-green-600 bg-white border-gray-200 rounded dark:border-gray-500 dark:bg-black accent-green-600 hover:bg-green-600 focus:ring-0"
            />
            <input
              required
              type="text"
              name="title"
              defaultValue={title}
              onChange={(e) => onTitleChange(e.target.value)}
              className="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
            />
          </fieldset>
          <Form method="post">
            <input type="hidden" name="id" value={id} hidden />
            <button
              type="submit"
              name="_action"
              value="todo-item-delete"
              aria-label="Delete todo"
              disabled={isDeleting}
              className="text-gray-400 transition hover:text-red-400"
            >
              <TrashIcon className="w-4 h-4" />
            </button>
          </Form>
        </div>
        <div className="flex justify-between mt-2 text-sm">
          <div
            className={`text-xs px-1 py-0.5 rounded ${
              completed
                ? 'bg-green-600 text-white'
                : 'bg-gray-300 dark:bg-gray-600'
            }`}
          >
            {completed ? 'Completed' : 'Not completed'}
          </div>
        </div>
      </div>
    </div>
  )
}

export default TodoItem
