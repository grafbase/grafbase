<script setup lang="ts">
import { useMutation } from "@urql/vue";
import {
  TodoFragment,
  TodoDeleteDocument,
  TodoUpdateDocument,
} from "@/graphql/schema";

interface Props {
  id: TodoFragment["id"];
  title: TodoFragment["title"];
  complete?: TodoFragment["complete"];
}

const todo = defineProps<Props>();
const titleRef = ref(todo.title);
const completed = ref(!!todo.complete);

const { executeMutation: todoDelete, fetching } =
  useMutation(TodoDeleteDocument);
const { executeMutation: todoUpdate } = useMutation(TodoUpdateDocument);

const onTodoUpdate = (newTodo: Partial<TodoFragment>) =>
  todoUpdate({ ...todo, ...newTodo });

const handleTodoDelete = () => {
  todoDelete({ id: todo.id }, { additionalTypenames: ["Todo"] });
};

watch(titleRef, (newValue) => {
  if (!newValue || newValue === todo.title) return;
  onTodoUpdate({ title: newValue });
});
watch(completed, (newValue) => {
  if (newValue === todo.complete) return;
  onTodoUpdate({ complete: newValue });
});
</script>

<template>
  <div
    class="relative p-3 overflow-hidden border rounded-md"
    :class="
      completed
        ? 'bg-emerald-200 dark:bg-emerald-800 border-emerald-600'
        : 'bg-zinc-50 dark:bg-zinc-800 border-gray-200 dark:border-transparent'
    "
  >
    <div
      v-if="completed"
      class="absolute inset-y-0 -inset-x-2.5 font-bold leading-[70px] text-black text-120px text-opacity-5"
    >
      DONE
    </div>
    <div class="relative">
      <div class="flex justify-between gap-4">
        <div :title="titleRef" class="flex space-x-1.5 items-center truncate">
          <input
            type="checkbox"
            v-model="completed"
            class="text-green-600 bg-white border-gray-200 rounded dark:border-gray-500 dark:bg-black accent-green-600 hover:bg-green-600 focus:ring-0"
          />
          <input
            v-model.trim.lazy="titleRef"
            class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
          />
        </div>
        <button
          @click="handleTodoDelete"
          class="text-gray-400 transition hover:text-red-400"
        >
          <Spinner v-if="fetching" />
          <IconHeroiconsOutlineTrash v-else class="w-4 h-4" />
        </button>
      </div>
      <div class="flex justify-between mt-2 text-sm">
        <div
          class="text-xs px-1 py-0.5 rounded"
          :class="
            completed
              ? 'bg-green-600 text-white'
              : 'bg-gray-300 dark:bg-gray-600'
          "
        >
          {{ completed ? "Completed" : "Not completed" }}
        </div>
      </div>
    </div>
  </div>
</template>
