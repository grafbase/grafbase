import { useTodoDeleteMutation } from "graphql/schema";
import { useMemo } from "react";

const TodoListTodo = (props: {
  title: string;
  id: string;
  complete?: boolean | null;
}) => {
  const { id, title, complete } = props;
  const contextDeleteTodoList = useMemo(
    () => ({ additionalTypenames: ["TodoList"] }),
    []
  );
  const [{ fetching }, todoDelete] = useTodoDeleteMutation();

  return (
    <div className=" rounded-lg border border-gray-200 p-4">
      <p className="font-medium text-gray-700">{title}</p>
      <div className="flex justify-between text-sm mt-2">
        <div className={`${complete ? "text-green-500" : "text-gray-500"}`}>
          {complete ? "Completed" : "Not Completed"}
        </div>
        <button
          onClick={() => todoDelete({ id }, contextDeleteTodoList)}
          className="text-xs text-gray-400 mt-0.5 hover:text-red-500"
        >
          {fetching ? "Deleting..." : "Delete"}
        </button>
      </div>
    </div>
  );
};

export default TodoListTodo;
