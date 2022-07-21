<script setup lang="ts">
import { useMutation } from "@urql/vue";
import { TodoCreateDocument } from "@/graphql/schema";

const { todoListId } = defineProps<{ todoListId: string }>();
const title = ref("");

const { executeMutation, fetching } = useMutation(TodoCreateDocument);

const handleTodoCreate = () => {
  executeMutation(
    { todoListId, title: title.value },
    { additionalTypenames: ["Todo"] }
  ).then(() => {
    title.value = "";
  });
};
</script>

<template>
  <form
    @submit.prevent="handleTodoCreate"
    class="flex items-center p-3 space-x-2 border-2 border-gray-200 border-dashed rounded-lg dark:border-gray-800"
  >
    <input
      required
      v-model="title"
      placeholder="Todo title"
      class="block w-full px-2 py-1 text-sm placeholder-gray-400 border border-gray-300 rounded-md bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:focus:ring-blue-500 dark:focus:border-blue-500"
    />
    <button
      :disabled="fetching"
      class="px-2 py-1 text-sm text-white bg-blue-800 rounded-md whitespace-nowrap disabled:bg-blue-400 min-w-[80px]"
    >
      {{ fetching ? "Adding..." : "Add Todo" }}
    </button>
  </form>
</template>
