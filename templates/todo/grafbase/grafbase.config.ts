import { g, config } from '@grafbase/sdk'

const todo = g.model('Todo', {
  list: g.relation(() => todoList),
  title: g.string(),
  complete: g.boolean().optional().default(false)
})

const todoList = g.model('TodoList', {
  title: g.string(),
  todos: g.relation(todo).optional().list().optional()
})

export default config({
  schema: g
})
