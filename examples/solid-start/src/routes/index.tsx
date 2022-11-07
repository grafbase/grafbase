import { For } from 'solid-js'
import { useRouteData } from 'solid-start'
import { createServerData$ } from 'solid-start/server'
import Layout from '~/components/layout'
import TodoList from '~/components/todo-list'
import TodoListCreate from '~/components/todo-list-create'
import { Query, TodoListsDocument } from '~/graphql/schema'
import { grafbase } from '~/utils/grafbase'

export const routeData = () => {
  return createServerData$(async (_, { request }) => {
    const data = await grafbase.request<Query>(TodoListsDocument)
    return data?.todoListCollection?.edges?.slice().reverse() ?? []
  })
}

const App = () => {
  const todoLists = useRouteData<typeof routeData>()

  return (
    <Layout>
      <div class='flex gap-6'>
        <For each={todoLists()}>{(list) => !!list?.node && <TodoList {...list.node} />}</For>
        <TodoListCreate />
      </div>
    </Layout>
  )
}

export default App
