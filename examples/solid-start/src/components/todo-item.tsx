import { createServerAction$, redirect } from 'solid-start/server'
import { TrashIcon } from '~/components/icons'
import {
  Mutation,
  TodoDeleteDocument,
  TodoUpdateDocument
} from '~/graphql/schema'
import { grafbase } from '~/utils/grafbase'

type Props = {
  id: string
  title: string
  complete: boolean
}

const TodoItem = (props: Props) => {
  const [deletingTodo, deleteTodo] = createServerAction$(
    async (form: FormData) => {
      const vars = Object.fromEntries(form)
      await grafbase.request<Mutation>(TodoDeleteDocument, vars)
      return redirect('/')
    }
  )

  const [updatingTodo, updateTodo] = createServerAction$(
    async (form: FormData) => {
      const vars = Object.fromEntries(form)
      await grafbase.request<Mutation>(TodoUpdateDocument, {
        ...vars,
        complete: vars.complete === 'on'
      })
      return redirect('/')
    }
  )

  const title = () =>
    updatingTodo.pending
      ? (updatingTodo.input.get('title') as string)
      : props.title
  const completed = () =>
    updatingTodo.pending
      ? updatingTodo.input.get('complete') === 'on'
      : props.complete

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
          <updateTodo.Form>
            <fieldset
              title={title()}
              disabled={updatingTodo.pending}
              class="flex space-x-1.5 items-center truncate"
            >
              <input type="hidden" name="id" value={props.id} hidden />
              <input
                type="checkbox"
                name="complete"
                checked={completed()}
                onChange={(e) => {
                  if (completed() !== e.currentTarget.checked) {
                    e.currentTarget.form.requestSubmit()
                  }
                }}
                class="text-green-600 bg-white border-gray-200 rounded dark:border-gray-500 dark:bg-black accent-green-600 hover:bg-green-600 focus:ring-0"
              />
              <input
                required
                type="text"
                name="title"
                value={title()}
                onChange={(e) => {
                  if (title() !== e.currentTarget.value) {
                    e.currentTarget.form.requestSubmit()
                  }
                }}
                class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
              />
            </fieldset>
          </updateTodo.Form>
          <deleteTodo.Form>
            <input type="hidden" name="id" value={props.id} hidden />
            <button
              type="submit"
              aria-label="Delete todo"
              disabled={deletingTodo.pending}
              class="text-gray-400 transition hover:text-red-400"
            >
              <TrashIcon class="w-4 h-4" />
            </button>
          </deleteTodo.Form>
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
