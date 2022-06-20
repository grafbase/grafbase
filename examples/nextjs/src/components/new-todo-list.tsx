import { useTodoListCreateMutation } from "graphql/schema";
import { useMemo, useState } from "react";

const NewTodoList = () => {
  const context = useMemo(() => ({ additionalTypenames: ["TodoList"] }), []);
  const [title, setTitle] = useState<string>("");

  const [{ fetching }, createTodoList] = useTodoListCreateMutation();

  return (
    <form
      className="h-fit rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3 space-y-3 min-w-[300px]"
      onSubmit={(e) => {
        e.preventDefault();
        createTodoList({ title }, context);
        setTitle("");
      }}
    >
      <h2 className="text-gray-900 dark:text-gray-300 font-semibold text-xl font-bold">
        New List
      </h2>
      <div className="flex space-x-3">
        <input
          required
          value={title}
          placeholder="Todo list title"
          onChange={(e) => setTitle(e.target.value)}
          className="w-[177px] bg-gray-50 px-2 py-1 dark:bg-zinc-800 dark:border-gray-800 placeholder-gray-400 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
        />
        <button
          disabled={fetching}
          className="bg-purple-600 px-5 text-sm rounded-md px-2 py-1 text-white disabled:bg-purple-500"
        >
          {fetching ? "Creating..." : "Create"}
        </button>
      </div>
    </form>
  );
};

export default NewTodoList;
