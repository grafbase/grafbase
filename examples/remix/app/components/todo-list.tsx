import { useFetcher } from '@remix-run/react'
import debounce from 'lodash.debounce'
import { useRef } from 'react'
import { Dropdown, DropdownItem } from '~/components/dropdown'
import { DotsVerticalIcon, PencilIcon, TrashIcon } from '~/components/icons'
import TodoItem from '~/components/todo-item'
import TodoItemCreate from '~/components/todo-item-create'
import type { TodoListFragment } from '~/graphql/schema'
import getColor from '~/utils/get-color'

const dropdownItemClass =
  'flex items-center w-full px-2 py-2 text-sm rounded-md group hover:bg-emerald-200 hover:dark:bg-emerald-700'

const TodoList = (props: TodoListFragment) => {
  const { id, title, todos } = props
  const { Form, submission, submit } = useFetcher()
  const inputRef = useRef<HTMLInputElement>(null)

  const isDeleting = submission?.formData.get('_action') === 'todo-list-delete'

  const onTodoListUpdate = (data: { title: string }) => {
    const form = new FormData()
    form.set('_action', 'todo-list-update')
    Object.entries({ id, ...data }).forEach(([key, value]) =>
      form.set(key, `${value}`)
    )
    return submit(form, { method: 'post' })
  }
  const onTitleChange = debounce(
    (title: string) => onTodoListUpdate({ title }),
    500
  )

  return (
    <div hidden={isDeleting} className="space-y-4 flex-1 min-w-[300px]">
      <div
        className="flex justify-between border-b-2"
        title={title || ''}
        style={{ borderColor: getColor(id) }}
      >
        <h2 className="text-xl font-bold truncate">
          <input
            ref={inputRef}
            required
            type="text"
            name="title"
            defaultValue={title}
            onChange={(e) => onTitleChange(e.target.value)}
            className="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
          />
        </h2>
        <div className="relative z-20">
          <Dropdown
            trigger={
              <DotsVerticalIcon className="w-5 h-5 text-gray-400 transition hover:text-red-400" />
            }
          >
            <DropdownItem>
              <button
                onClick={() => setTimeout(() => inputRef.current?.focus(), 100)}
                className={dropdownItemClass}
              >
                <PencilIcon className="w-5 h-5 mr-3" aria-hidden="true" />
                Rename
              </button>
            </DropdownItem>
            <DropdownItem>
              <Form method="post">
                <input type="hidden" name="id" value={id} hidden />
                <button
                  type="submit"
                  name="_action"
                  value="todo-list-delete"
                  aria-label="Delete todo"
                  disabled={isDeleting}
                  className={dropdownItemClass}
                >
                  <TrashIcon className="w-5 h-5 mr-3" aria-hidden="true" />
                  Delete
                </button>
              </Form>
            </DropdownItem>
          </Dropdown>
        </div>
      </div>
      <div className="space-y-4">
        {todos?.map((todo) => !!todo && <TodoItem key={todo.id} {...todo} />)}
      </div>
      <TodoItemCreate todoListId={id} />
    </div>
  )
}

export default TodoList
