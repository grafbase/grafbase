import Head from "next/head";

const features = [
  {
    title: "Auth",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
  {
    title: "Branching",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
  {
    title: "CLI",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
  {
    title: "Edge",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
  {
    title: "Migrations",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
  {
    title: "Resolvers",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
];

const tecnologyUsed = [
  {
    title: "Grafbase",
    description: "Lorem ipsum dolor sir amet",
  },
  {
    title: "Clerk",
    description: "Lorem ipsum dolor sir amet",
    link: "https://grafbase.com",
  },
  {
    title: "Apollo",
    description: "Lorem ipsum dolor sir amet",
  },
  {
    title: "Tailwind",
    description: "Lorem ipsum dolor sir amet",
  },
];

const AboutPage = () => {
  return (
    <div>
      <Head>
        <title>About | Grafnews</title>
      </Head>
      <h1 className="text-5xl font-bold">
        Everything you need to develop a GraphQL app
      </h1>
      <div className="border-b-4 mt-6 max-w-sm border-black" />
      <p className="text-xl mt-4 text-gray-600">
        Build your next GraphQL powered application faster and easier with
        Grafbase and Clerk.
      </p>
      <h3 className="mt-8 text-2xl font-semibold">Features</h3>
      <div className="grid grid-cols-2 sm:grid-cols-3 mt-14 gap-12">
        {features.map(({ title, description, link }) => (
          <a
            key={title}
            href={link}
            target="_blank"
            rel="noreferrer"
            className="flex flex-col items-center justify-center relative border border-black p-4 group"
          >
            <div className="absolute border-b-4 border-b-gray-300 flex items-center justify-center text-xl font-bold text-white -top-8 bg-black w-16 h-16 group-hover:bg-indigo-800">
              {title[0]}
            </div>
            <h3 className="mt-8 text-2xl text-center">{title}</h3>
            <p className="mt-3 text-center text-gray-600">{description}</p>
          </a>
        ))}
      </div>
      <h3 className="mt-12 text-2xl font-semibold">Technology used</h3>
      <div className="grid grid-cols-2 sm:grid-cols-3 mt-8 gap-8">
        {tecnologyUsed.map(({ title, description }) => (
          <div key={title} className="bg-gray-50 justify-center p-4 group">
            <h3 className="text-xl">{title}</h3>
            <p className="mt-3 text-gray-600">{description}</p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default AboutPage;
