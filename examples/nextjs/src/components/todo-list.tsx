import { TodoListFragment, TodoListDeleteDocument } from "graphql/schema";
import { useMutation } from "urql";
import { useMemo } from "react";
import TodoListCreateTodo from "components/todo-list.create-todo";
import TodoListTodo from "components/todo-list.todo";
import { TrashIcon } from "@heroicons/react/outline";
import getColor from "utils/get-color";
import Spinner from "components/spinner";

const TodoList = (props: TodoListFragment) => {
  const { id, title, todos } = props;
  const contextDeleteTodoList = useMemo(
    () => ({ additionalTypenames: ["TodoList", "Todo"] }),
    []
  );
  const [{ fetching }, todoListDelete] = useMutation(TodoListDeleteDocument);

  return (
    <div className="space-y-4 flex-1 min-w-[300px]">
      <div
        className="flex justify-between border-b-2 truncate"
        title={title}
        style={{ borderColor: getColor(id) }}
      >
        <h2 className="font-bold text-xl truncate">{title}</h2>
        <button
          className="text-gray-400 hover:text-red-400 transition"
          onClick={() => todoListDelete({ id }, contextDeleteTodoList)}
        >
          {fetching ? <Spinner /> : <TrashIcon className="w-4 h-4" />}
        </button>
      </div>
      <div className="space-y-4">
        {todos?.map(
          (todo) => !!todo && <TodoListTodo key={todo.id} {...todo} />
        )}
      </div>
      <TodoListCreateTodo todoListId={id} />
    </div>
  );
};

export default TodoList;
