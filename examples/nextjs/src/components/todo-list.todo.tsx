import {
  TodoDeleteDocument,
  TodoFragment,
  TodoUpdateDocument,
} from "graphql/schema";
import { useMutation } from "urql";
import { useMemo, useState } from "react";
import { TrashIcon } from "@heroicons/react/outline";
import Spinner from "components/spinner";
import debounce from "lodash.debounce";

const TodoListTodo = (props: {
  title: string;
  id: string;
  complete?: boolean | null;
}) => {
  const { id, title, complete } = props;
  const [completed, setCompleted] = useState(!!complete);
  const contextDeleteTodoList = useMemo(
    () => ({ additionalTypenames: ["TodoList"] }),
    []
  );
  const [{ fetching }, todoDelete] = useMutation(TodoDeleteDocument);
  const [{}, todoUpdate] = useMutation(TodoUpdateDocument);

  const onTodoUpdate = (todoProps: Partial<TodoFragment>) =>
    todoUpdate({ ...props, ...todoProps });

  const onTitleChange = debounce((title: string) => {
    onTodoUpdate({ title });
  }, 500);

  return (
    <div
      className={`relative rounded-md border p-3 overflow-hidden ${
        completed
          ? "bg-emerald-800 border-emerald-600"
          : "bg-zinc-50 dark:bg-zinc-800 border-gray-200 dark:border-transparent"
      }`}
    >
      {completed && (
        <div className="absolute text-8xl font-bold left-0 -top-3 text-white text-opacity-5 tracking-wider">
          DONE
        </div>
      )}
      <div className="relative">
        <div className="flex justify-between gap-4">
          <div className="flex space-x-1.5 items-center truncate" title={title}>
            <input
              type="checkbox"
              defaultChecked={completed}
              className="border-gray-200 text-green-600 dark:border-gray-500 bg-white dark:bg-black rounded accent-green-600 hover:bg-green-600 focus:ring-0"
              onClick={() => {
                setCompleted((c) => {
                  onTodoUpdate({ complete: !c });
                  return !c;
                });
              }}
            />
            <input
              defaultValue={title}
              className={`bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400 ${
                completed ? "text-white" : "text-black dark:text-white"
              }`}
              onChange={(e) => onTitleChange(e?.target?.value)}
            />
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
              completed
                ? "bg-green-600 text-white"
                : "bg-gray-300 dark:bg-gray-600 text-black dark:text-white"
            }`}
          >
            {completed ? "Completed" : "Not completed"}
          </div>
        </div>
      </div>
    </div>
  );
};

export default TodoListTodo;
