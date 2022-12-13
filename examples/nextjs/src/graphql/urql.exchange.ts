import { gql } from 'urql'
import { cacheExchange as graphCache } from '@urql/exchange-graphcache'
import {
  Todo,
  TodoCreateMutation,
  TodoListConnection,
  TodoListCreateMutation
} from 'graphql/schema'

const TodoCollectionList = gql`
  {
    todoListCollection(first: 100) {
      edges {
        node {
          id
          title
          todos {
            id
            title
            complete
          }
        }
      }
    }
  }
`

export const cacheExchange = () =>
  graphCache({
    updates: {
      Mutation: {
        todoCreate(
          result: TodoCreateMutation,
          _args: { input: { list: { link: string } } },
          cache,
          _info
        ) {
          cache.updateQuery(
            { query: TodoCollectionList },
            (data: { todoListCollection: TodoListConnection } | null) => {
              data?.todoListCollection?.edges
                ?.find((edge) => edge?.node?.id === _args?.input?.list?.link)
                ?.node?.todos?.push(result?.todoCreate?.todo as Todo)

              return data
            }
          )
        },
        todoListCreate(result: TodoListCreateMutation, _args, cache, _info) {
          cache.updateQuery({ query: TodoCollectionList }, (data) => {
            data.todoListCollection.edges = [
              {
                node: result.todoListCreate?.todoList,
                __typename: 'TodoListEdge'
              },
              ...data.todoListCollection.edges
            ]

            return data
          })
        },
        todoListDelete(result, _args, cache, _info) {
          cache
            .inspectFields('Query')
            .filter((field) => field.fieldName === 'todoListCollection')
            .forEach(() => {
              cache.updateQuery(
                {
                  query: TodoCollectionList
                },
                (data: { todoListCollection: TodoListConnection } | null) => {
                  // @ts-ignore
                  data.todoListCollection.edges =
                    data?.todoListCollection.edges?.filter(
                      (edge) => edge?.node?.id !== _args.id
                    )
                  return data
                }
              )
            })
        },
        todoDelete(result, _args, cache, _info) {
          cache
            .inspectFields('Query')
            .filter((field) => field.fieldName === 'todoListCollection')
            .forEach(() => {
              cache.updateQuery(
                {
                  query: TodoCollectionList
                },
                (data: { todoListCollection: TodoListConnection } | null) => {
                  // @ts-ignore
                  data.todoListCollection.edges =
                    data?.todoListCollection.edges?.map((edge) => {
                      // @ts-ignore
                      edge.node.todos = edge?.node?.todos.filter(
                        (todo) => todo?.id !== _args.id
                      )

                      return edge
                    })
                  return data
                }
              )
            })
        }
      }
    }
  })
