import debounce from 'lodash.debounce'
import { createSignal } from 'solid-js'
import { createMutation } from 'solid-urql'

import { SpinnerIcon, TrashIcon } from '~/components/icons'
import { TodoDeleteDocument, TodoUpdateDocument } from '~/graphql/schema'

const TodoDeleteContext = { additionalTypenames: ['TodoList'] }

type Props = {
  id: string
  title: string
  complete: boolean
}

const TodoItem = (props: Props) => {
  const { id, title, complete } = props
  const [completed, setCompleted] = createSignal(!!complete)

  const [deleting, todoDelete] = createMutation(TodoDeleteDocument)
  const [_, todoUpdate] = createMutation(TodoUpdateDocument)

  const onTodoUpdate = (data: Partial<Props>) =>
    todoUpdate({ ...props, ...data })
  const onTitleChange = debounce(
    (title: string) => onTodoUpdate({ title }),
    500
  )
  const onChecked = (c: boolean) => {
    setCompleted(c)
    onTodoUpdate({ complete: c })
  }

  return (
    <div
      class={`relative p-3 overflow-hidden border rounded-md ${
        completed()
          ? 'bg-emerald-200 dark:bg-emerald-800 border-emerald-600'
          : 'bg-zinc-50 dark:bg-gray-700 border-gray-200 dark:border-transparent'
      }`}
    >
      {completed() && (
        <div class="absolute inset-y-0 -inset-x-2.5 font-bold leading-[70px] text-black text-[120px] text-opacity-5 uppercase">
          done
        </div>
      )}
      <div class="relative">
        <div class="flex justify-between gap-4">
          <fieldset
            title={title}
            class="flex space-x-1.5 items-center truncate"
          >
            <input
              type="checkbox"
              name="complete"
              checked={completed()}
              onChange={(e) => onChecked(e.currentTarget.checked)}
              class="text-green-600 bg-white border-gray-200 rounded dark:border-gray-500 dark:bg-black accent-green-600 hover:bg-green-600 focus:ring-0"
            />
            <input
              required
              type="text"
              name="title"
              value={title}
              onChange={(e) => onTitleChange(e.currentTarget.value)}
              class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
            />
          </fieldset>
          <button
            aria-label="Delete todo"
            disabled={deleting().fetching}
            class="text-gray-400 transition hover:text-red-400"
            onClick={() => todoDelete({ id }, TodoDeleteContext)}
          >
            {deleting().fetching ? (
              <SpinnerIcon />
            ) : (
              <TrashIcon class="w-4 h-4" />
            )}
          </button>
        </div>
        <div class="flex justify-between mt-2 text-sm">
          <div
            class={`text-xs px-1 py-0.5 rounded ${
              completed()
                ? 'bg-green-600 text-white'
                : 'bg-gray-300 dark:bg-gray-600'
            }`}
          >
            {completed() ? 'Completed' : 'Not completed'}
          </div>
        </div>
      </div>
    </div>
  )
}

export default TodoItem
