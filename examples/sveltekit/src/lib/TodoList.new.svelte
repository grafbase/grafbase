<script lang="ts">
  import { getContextClient, mutationStore } from '@urql/svelte'
  import { TodoListCreateDocument } from '$graphql/schema'

  let title = ''
  let client = getContextClient()
  async function submit() {
    const result = await mutationStore({
      client,
      query: TodoListCreateDocument,
      variables: { title }
    })
    console.log(result)
    title = ''
  }
</script>

<form
  on:submit|preventDefault={submit}
  class="h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]"
>
  <h2 class="text-gray-900 dark:text-gray-300 text-xl font-bold">New List</h2>
  <div class="flex space-x-3">
    <input
      bind:value={title}
      required
      placeholder="Todo list title"
      class="bg-gray-50 px-2 py-1 placeholder-gray-400 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
    />
    <button
      class="bg-purple-600 text-sm rounded-md px-2 py-1 text-white disabled:bg-purple-500"
    >
      Create
    </button>
  </div>
</form>
