import debounce from 'lodash.debounce'
import { For, Match, Switch } from 'solid-js'
import { createMutation } from 'solid-urql'

import { Dropdown, DropdownItem } from '~/components/dropdown'
import {
  DotsVerticalIcon,
  PencilIcon,
  SpinnerIcon,
  TrashIcon
} from '~/components/icons'
import TodoItem from '~/components/todo-item'
import TodoItemCreate from '~/components/todo-item-create'
import {
  TodoListDeleteDocument,
  TodoListFragment,
  TodoListUpdateDocument
} from '~/graphql/schema'
import getColor from '~/utils/get-color'

const TodoListDeleteContext = { additionalTypenames: ['TodoList', 'Todo'] }

const TodoList = (props: TodoListFragment) => {
  const { id, title, todos } = props
  let inputRef: HTMLInputElement

  const [deleting, todoListDelete] = createMutation(TodoListDeleteDocument)
  const [_, todoListUpdate] = createMutation(TodoListUpdateDocument)

  const onTitleChange = debounce((title: string) => {
    todoListUpdate({ id, title })
  }, 500)

  return (
    <div class="space-y-4 flex-1 min-w-[300px]">
      <div
        class="flex justify-between border-b-2"
        title={title || ''}
        style={{ borderColor: getColor(id) }}
      >
        <h2 class="text-xl font-bold truncate">
          <input
            ref={(el) => (inputRef = el)}
            required
            type="text"
            name="title"
            value={title}
            onChange={(e) => onTitleChange(e.currentTarget.value)}
            class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
          />
        </h2>
        <div class="relative z-20">
          <Switch>
            <Match when={deleting().fetching}>
              <SpinnerIcon />
            </Match>
            <Match when={!deleting().fetching}>
              <Dropdown
                trigger={
                  <DotsVerticalIcon class="w-5 h-5 text-gray-400 transition hover:text-red-400" />
                }
              >
                <DropdownItem>
                  <button
                    onClick={() => setTimeout(() => inputRef.focus(), 100)}
                    class="flex items-center w-full px-2 py-2 text-sm rounded-md group hover:bg-emerald-200 hover:dark:bg-emerald-700"
                  >
                    <PencilIcon class="w-5 h-5 mr-3" aria-hidden="true" />
                    Rename
                  </button>
                </DropdownItem>
                <DropdownItem>
                  <button
                    disabled={deleting().fetching}
                    onClick={() =>
                      todoListDelete({ id }, TodoListDeleteContext)
                    }
                    class="flex items-center w-full px-2 py-2 text-sm rounded-md group hover:bg-emerald-200 hover:dark:bg-emerald-700"
                  >
                    <TrashIcon class="w-5 h-5 mr-3" aria-hidden="true" />
                    Delete
                  </button>
                </DropdownItem>
              </Dropdown>
            </Match>
          </Switch>
        </div>
      </div>
      <div class="space-y-4">
        <For each={todos}>{(todo) => !!todo && <TodoItem {...todo} />}</For>
      </div>
      <TodoItemCreate todoListId={id} />
    </div>
  )
}

export default TodoList
