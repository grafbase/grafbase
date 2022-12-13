<script lang="ts">
  import getColor from '$lib/utils/get-color'
  import Todo from '$lib/Todo.svelte'
  import Menu from '$lib/Menu.svelte'
  import TodoNew from '$lib/Todo.New.svelte'

  import type { TodoList } from '$graphql/schema'
  import { getContextClient, mutationStore } from '@urql/svelte'
  import {
    TodoListDeleteDocument,
    TodoListUpdateDocument
  } from '$graphql/schema'

  export let todoList: TodoList

  let input: HTMLInputElement
  let client = getContextClient()

  let title
  let timer

  const debounce = (v) => {
    clearTimeout(timer)
    timer = setTimeout(() => {
      title = v
    }, 750)
  }

  function updateTodoList(title: string) {
    mutationStore({
      client,
      query: TodoListUpdateDocument,
      variables: { id: todoList.id, title }
    })
  }

  $: {
    if (title) {
      updateTodoList(title)
    }
  }

  function deleteTodoList() {
    mutationStore({
      client,
      query: TodoListDeleteDocument,
      variables: { id: todoList.id }
    })
  }

  let menuOptions = [
    {
      name: 'Rename',
      onClick: () => setTimeout(() => input.focus(), 10)
    },
    {
      name: 'Delete',
      onClick: deleteTodoList
    }
  ]
</script>

<div class="space-y-4 flex-1 min-w-[300px]">
  <div
    class="flex justify-between border-b-2 "
    title={todoList.title}
    style={`border-color: ${getColor(todoList.id)}`}
  >
    <h2 class="text-xl font-bold truncate">
      <input
        bind:this={input}
        on:keyup={({ target: { value } }) => debounce(value)}
        value={todoList.title}
        class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
      />
    </h2>
    <div class="relative z-20">
      <Menu options={menuOptions}>
        <svg
          class="w-5 h-5 text-gray-600 transition hover:text-blue-600"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
          />
        </svg>
      </Menu>
    </div>
  </div>
  <div class="space-y-4">
    {#if todoList?.todos?.edges?.length}
      {#each todoList.todos.edges as edge}
        {#if edge?.node}
          <Todo todo={edge?.node} />
        {/if}
      {/each}
    {/if}
    <TodoNew todoListId={todoList.id} />
  </div>
</div>
