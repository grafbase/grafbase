<script setup lang="ts">
import { useMutation } from '@urql/vue'
import { TodoListCreateDocument } from '@/graphql/schema'

const title = ref('')
const { fetching, executeMutation } = useMutation(TodoListCreateDocument)

const handleTodoListCreate = () => {
  executeMutation(
    { title: title.value },
    { additionalTypenames: ['TodoList'] }
  ).then(() => {
    title.value = ''
  })
}
</script>

<template>
  <form
    @submit.prevent="handleTodoListCreate"
    class="h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]"
  >
    <h2 class="text-xl font-bold text-gray-900 dark:text-gray-300">New List</h2>
    <div class="flex space-x-3">
      <input
        required
        v-model="title"
        placeholder="Todo list title"
        class="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
      />
      <button
        :disabled="fetching"
        class="px-5 py-1 text-sm text-white bg-purple-600 rounded-md disabled:bg-purple-500 min-w-[110px]"
      >
        {{ fetching ? 'Creating...' : 'Create' }}
      </button>
    </div>
  </form>
</template>
