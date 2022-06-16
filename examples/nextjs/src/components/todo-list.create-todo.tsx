import { useCreateTodoMutation } from "graphql/schema";
import { useMemo, useState } from "react";

const TodoListCreateTodo = ({ todoListId }: { todoListId: string }) => {
  const context = useMemo(() => ({ additionalTypenames: ["Todo"] }), []);

  const [title, setTitle] = useState<string>("");

  const [{ fetching }, createTodo] = useCreateTodoMutation();

  return (
    <form
      className="h-fit space-y-2 rounded-lg border border-dashed border-gray-200 p-4 hover:text-grafbase"
      onSubmit={(e) => {
        e.preventDefault();
        createTodo({ title, todoListId }, context);
        setTitle("");
      }}
    >
      <input
        required
        value={title}
        placeholder="Type todo title"
        onChange={(e) => setTitle(e.target.value)}
        className="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
      />
      <button
        disabled={fetching}
        className="bg-blue-400 rounded-md px-2 py-1 text-white w-full"
      >
        {fetching ? "Adding..." : "Add"}
      </button>
    </form>
  );
};

export default TodoListCreateTodo;
