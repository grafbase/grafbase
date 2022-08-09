<script setup lang="ts">
import { useMutation } from '@urql/vue'
import {
  TodoListFragment,
  TodoListDeleteDocument,
  TodoListUpdateDocument
} from '@/graphql/schema'

interface Props {
  id: TodoListFragment['id']
  title: TodoListFragment['title']
  todos?: TodoListFragment['todos']
}

const { id, title, todos } = defineProps<Props>()
const titleRef = ref(title)
const inputRef = ref<HTMLElement>()

const todoListUpdate = useMutation(TodoListUpdateDocument)
const { executeMutation: todoListDelete, fetching: isDeleting } = useMutation(
  TodoListDeleteDocument
)

const handleTodoListDelete = () => {
  todoListDelete({ id }, { additionalTypenames: ['TodoList'] })
}

watch(titleRef, (newValue) => {
  if (!newValue || newValue === title) return
  todoListUpdate.executeMutation({ id, title: newValue })
})
</script>

<template>
  <div class="space-y-4 flex-1 min-w-[300px]">
    <div
      :title="titleRef"
      :style="{ borderColor: getColor(id) }"
      class="flex justify-between border-b-2"
    >
      <h2 class="text-xl font-bold truncate">
        <input
          ref="inputRef"
          v-model.trim.lazy="titleRef"
          class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
        />
      </h2>
      <div class="relative z-20">
        <Spinner v-if="isDeleting" />
        <Dropdown v-else>
          <template #trigger>
            <IconHeroiconsOutlineDotsVertical
              class="w-5 h-5 text-gray-400 transition hover:text-red-400"
            />
          </template>
          <DropdownItem @click="() => nextTick(() => inputRef?.focus())">
            <IconHeroiconsOutlinePencil
              class="w-5 h-5 mr-3"
              aria-hidden="true"
            />
            Rename
          </DropdownItem>
          <DropdownItem @click="handleTodoListDelete">
            <IconHeroiconsOutlineTrash
              class="w-5 h-5 mr-3"
              aria-hidden="true"
            />
            Delete
          </DropdownItem>
        </Dropdown>
      </div>
    </div>
    <div v-if="todos" class="space-y-4">
      <div v-for="todo in todos" :key="todo?.id">
        <TodoItem v-if="todo?.id" v-bind="todo" />
      </div>
    </div>
    <TodoItemCreate :todoListId="id" />
  </div>
</template>
