import { TodoCreateDocument } from "graphql/schema";
import { useMutation } from "urql";
import { useMemo, useState } from "react";

const TodoListCreateTodo = ({ todoListId }: { todoListId: string }) => {
  const context = useMemo(() => ({ additionalTypenames: ["Todo"] }), []);

  const [title, setTitle] = useState<string>("");

  const [{ fetching }, createTodo] = useMutation(TodoCreateDocument);

  return (
    <form
      className="flex items-center space-x-2 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3"
      onSubmit={(e) => {
        e.preventDefault();
        createTodo({ title, todoListId }, context);
        setTitle("");
      }}
    >
      <input
        required
        value={title}
        placeholder="Todo title"
        onChange={(e) => setTitle(e.target.value)}
        className="w-[177px] bg-gray-50 dark:bg-zinc-800 px-2 py-1 text-sm border border-gray-300 dark:border-gray-800 text-gray-900 text-sm rounded-md focus:ring-blue-500 focus:border-blue-500 block w-full placeholder-gray-400 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
      />
      <button
        disabled={fetching}
        className="bg-blue-800 text-sm rounded-md px-2 py-1 text-white whitespace-nowrap disabled:bg-blue-400"
      >
        {fetching ? "Adding..." : "Add Todo"}
      </button>
    </form>
  );
};

export default TodoListCreateTodo;
