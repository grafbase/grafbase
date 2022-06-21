import { TodoDeleteDocument } from "graphql/schema";
import { useMutation } from "urql";
import { useMemo } from "react";
import { TrashIcon } from "@heroicons/react/outline";
import Spinner from "components/spinner";

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
  const [{ fetching }, todoDelete] = useMutation(TodoDeleteDocument);

  return (
    <div className="rounded-md border border-gray-200 dark:border-gray-700 p-3 bg-zinc-50 dark:bg-zinc-900">
      <div className="flex justify-between gap-4">
        <div className="flex space-x-1.5 items-center truncate" title={title}>
          <input
            type="checkbox"
            className="border-gray-200 dark:border-gray-500 bg-white dark:bg-black rounded"
          />
          <p className="font-semibold text-black dark:text-white text-sm truncate">
            {title}
          </p>
        </div>
        <button
          className="text-gray-400 hover:text-red-400 transition"
          onClick={() => todoDelete({ id }, contextDeleteTodoList)}
        >
          {fetching ? <Spinner /> : <TrashIcon className="w-4 h-4" />}
        </button>
      </div>
      <div className="flex justify-between text-sm mt-2">
        <div
          className={`text-xs px-1 py-0.5 rounded ${
            complete
              ? "bg-green-800 text-white"
              : "bg-gray-300 dark:bg-gray-600 text-black dark:text-white"
          }`}
        >
          {complete ? "Completed" : "Not completed"}
        </div>
      </div>
    </div>
  );
};

export default TodoListTodo;
