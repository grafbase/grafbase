import type { NextPage } from "next";
import { getSession } from "next-auth/react";
import { NextPageContext } from "next";
import { useTodosQuery } from "graphql/schema";
import TodoList from "components/todo-list";
import TodoListEmpty from "components/todo-list-empty";

const Home: NextPage = () => {
  const [{ data, fetching }] = useTodosQuery();

  if (fetching) {
    return <div>Loading...</div>;
  }

  return (
    <div className="grid grid-cols-5 gap-6">
      {data?.todoListCollection?.edges?.reverse().map((todoList, index) => {
        if (!todoList?.node) {
          return null;
        }

        return <TodoList key={index} {...todoList.node} />;
      })}
      <TodoListEmpty />
    </div>
  );
};

export async function getServerSideProps(context: NextPageContext) {
  const session = await getSession(context);

  if (!session) {
    return {
      redirect: {
        destination: "/sign-in",
        permanent: false,
      },
    };
  }

  return {
    props: {
      session,
    },
  };
}

export default Home;
