import { For } from 'solid-js'
import { createServerAction$, redirect } from 'solid-start/server'
import { Dropdown, DropdownItem } from '~/components/dropdown'
import { DotsVerticalIcon, PencilIcon, TrashIcon } from '~/components/icons'
import TodoItem from '~/components/todo-item'
import TodoItemCreate from '~/components/todo-item-create'
import {
  Mutation,
  TodoListDeleteDocument,
  TodoListFragment,
  TodoListUpdateDocument
} from '~/graphql/schema'
import getColor from '~/utils/get-color'
import { grafbase } from '~/utils/grafbase'

const TodoList = (props: TodoListFragment) => {
  const [deletingTodoList, deleteTodoList] = createServerAction$(
    async (form: FormData) => {
      const vars = Object.fromEntries(form)
      await grafbase.request<Mutation>(TodoListDeleteDocument, vars)
      return redirect('/')
    }
  )

  const [updatingTodoList, updateTodoList] = createServerAction$(
    async (form: FormData) => {
      const vars = Object.fromEntries(form)
      await grafbase.request<Mutation>(TodoListUpdateDocument, vars)
      return redirect('/')
    }
  )

  const todos = () => props.todos.edges
  const title = () =>
    updatingTodoList.pending
      ? (updatingTodoList.input.get('title') as string)
      : props.title
  let inputRef: HTMLInputElement

  return (
    <div class="space-y-4 flex-1 min-w-[300px]">
      <div
        class="flex justify-between border-b-2"
        title={title() || ''}
        style={{ 'border-color': getColor(props.id) }}
      >
        <updateTodoList.Form>
          <input type="hidden" name="id" value={props.id} hidden />
          <h2 class="text-xl font-bold truncate">
            <input
              ref={inputRef}
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
          </h2>
        </updateTodoList.Form>
        <div class="relative z-20">
          <Dropdown
            trigger={
              <DotsVerticalIcon class="w-5 h-5 text-gray-400 transition hover:text-red-400" />
            }
          >
            <DropdownItem>
              <div class="group px-2 py-2 rounded-md hover:bg-emerald-200 hover:dark:bg-emerald-700">
                <button
                  onClick={() => setTimeout(() => inputRef.focus())}
                  class="flex items-center w-full text-sm"
                >
                  <PencilIcon class="w-5 h-5 mr-3" aria-hidden="true" />
                  Rename
                </button>
              </div>
            </DropdownItem>
            <DropdownItem>
              <deleteTodoList.Form class="group px-2 py-2 rounded-md hover:bg-emerald-200 hover:dark:bg-emerald-700">
                <input type="hidden" name="id" value={props.id} hidden />
                <button
                  type="submit"
                  aria-label="Delete todo list"
                  disabled={deletingTodoList.pending}
                  class="flex items-center w-full text-sm"
                >
                  <TrashIcon class="w-5 h-5 mr-3" aria-hidden="true" />
                  Delete
                </button>
              </deleteTodoList.Form>
            </DropdownItem>
          </Dropdown>
        </div>
      </div>
      <div class="space-y-4">
        <For each={todos()}>
          {(todo) => !!todo?.node && <TodoItem {...todo.node} />}
        </For>
      </div>
      <TodoItemCreate todoListId={props.id} />
    </div>
  )
}

export default TodoList
