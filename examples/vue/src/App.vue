<template>
  <div class="app">
    <h3>Messages:</h3>
    <ul>
      <li v-for="node in nodes" :key="node.id">
        <p>{{ node.author }}</p>
        <p>{{ node.body }}</p>
        <p>{{ node.createdAt }}</p>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'

interface MessageNode {
  id: string
  author: string
  body: string
  createdAt: string
}

export default defineComponent({
  name: 'App',

  setup() {
    const nodes = ref<MessageNode[]>([])

    const GetAllMessagesQuery = /* GraphQL */ `
      query GetAllMessages($first: Int!) {
        messageCollection(first: $first) {
          edges {
            node {
              id
              author
              body
              createdAt
            }
          }
        }
      }
    `

    fetch(process.env.VUE_APP_GRAFBASE_API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        query: GetAllMessagesQuery,
        variables: {
          first: 100
        }
      })
    })
      .then((res) => res.json())
      .then((data) => {
        nodes.value = data.data.messageCollection.edges.map(
          (edge: { node: MessageNode }) => edge.node
        )
      })
      .catch((error) => {
        console.error(error)
      })

    return { nodes }
  }
})
</script>
