import type { NextPage } from 'next'
import { useQuery } from 'urql'
import { TodoListsDocument } from 'graphql/schema'
import TodoList from 'components/todo-list'
import TodoListEmpty from 'components/new-todo-list'
import { useMemo } from 'react'

const Home: NextPage = () => {
  const context = useMemo(
    () => ({ additionalTypenames: ['TodoList', 'Todo'] }),
    []
  )

  const [{ data, fetching }] = useQuery({ query: TodoListsDocument, context })

  const reversed = useMemo(() => {
    if (!data?.todoListCollection?.edges) {
      return
    }

    return [...data?.todoListCollection?.edges].reverse()
  }, [data])

  if (fetching) {
    return <div>Loading...</div>
  }

  return (
    <div className="flex gap-6">
      {reversed?.map((todoList, index) => {
        if (!todoList?.node) {
          return null
        }

        return <TodoList key={index} {...todoList.node} />
      })}
      <TodoListEmpty />
    </div>
  )
}

export default Home
