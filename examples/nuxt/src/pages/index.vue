<script setup lang="ts">
import { useQuery } from '@urql/vue'
import { TodoListsDocument } from '@/graphql/schema'

const { data, fetching, error } = useQuery({ query: TodoListsDocument })
</script>

<template>
  <div>
    <div v-if="fetching">Loading...</div>
    <div v-else-if="error">{{ error }}</div>
    <div v-else-if="data" class="flex gap-6">
      <div
        v-for="list in data.todoListCollection?.edges?.slice().reverse()"
        :key="list?.node?.id"
      >
        <TodoList v-if="list?.node?.id" v-bind="list.node" />
      </div>
      <TodoListCreate />
    </div>
  </div>
</template>
