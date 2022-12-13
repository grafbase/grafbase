import type {
  ActionFunction,
  LoaderFunction,
  MetaFunction
} from '@remix-run/node'
import { json } from '@remix-run/node'
import { useLoaderData } from '@remix-run/react'
import TodoList from '~/components/todo-list'
import TodoListCreate from '~/components/todo-list-create'
import type { Maybe, Mutation, Query, TodoListEdge } from '~/graphql/schema'
import {
  TodoCreateDocument,
  TodoDeleteDocument,
  TodoListCreateDocument,
  TodoListDeleteDocument,
  TodoListsDocument,
  TodoListUpdateDocument,
  TodoUpdateDocument
} from '~/graphql/schema'
import { client } from '~/utils/graphql.server'

export const meta: MetaFunction = () => ({
  title: 'Remix - Todo Example - Grafbase',
  description: 'Todo Example leveraging the Grafbase platform'
})

export const action: ActionFunction = async ({ request }) => {
  const form = await request.formData()
  const { _action, ...vars } = Object.fromEntries(form)

  if (_action === 'todo-list-create') {
    return await client.request<Mutation>(TodoListCreateDocument, vars)
  }
  if (_action === 'todo-list-delete') {
    return await client.request<Mutation>(TodoListDeleteDocument, vars)
  }
  if (_action === 'todo-list-update') {
    return await client.request<Mutation>(TodoListUpdateDocument, vars)
  }
  if (_action === 'todo-item-create') {
    return await client.request<Mutation>(TodoCreateDocument, vars)
  }
  if (_action === 'todo-item-delete') {
    return await client.request<Mutation>(TodoDeleteDocument, vars)
  }
  if (_action === 'todo-item-update') {
    return await client.request<Mutation>(TodoUpdateDocument, {
      ...vars,
      complete: vars.complete === 'true'
    })
  }

  return null
}

export const loader: LoaderFunction = async () => {
  const data = await client.request<Query>(TodoListsDocument)
  const todoLists = data.todoListCollection?.edges?.slice().reverse() ?? []
  return json(todoLists)
}

export default function Index() {
  const todoLists = useLoaderData<Maybe<TodoListEdge>[]>()

  return (
    <div className="flex gap-6">
      {todoLists.map(
        (list) => !!list?.node && <TodoList key={list.node.id} {...list.node} />
      )}
      <TodoListCreate />
    </div>
  )
}
