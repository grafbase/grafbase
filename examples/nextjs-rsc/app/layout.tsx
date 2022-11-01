import './globals.css'

import Link from 'next/link'

const GetAllPostsQuery = /* GraphQL */ `
  query GetAllPosts($first: Int!) {
    postCollection(first: $first) {
      edges {
        node {
          id
          title
          slug
        }
      }
    }
  }
`

const getPosts = async () => {
  const res = await fetch(process.env.NEXT_PUBLIC_GRAFBASE_API_URL!, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
      // 'x-api-key': '...'
    },
    body: JSON.stringify({ query: GetAllPostsQuery, variables: { first: 50 } })
  })

  return res.json()
}

const RootLayout = async ({ children }: { children: React.ReactNode }) => {
  const { data } = await getPosts()

  return (
    <html lang="en">
      <head>
        <title>Grafbase!</title>
      </head>
      <body>
        <div className="flex space-x-8">
          <nav className="w-[400px] flex flex-col justify-between h-screen overflow-y-auto bg-gray-100">
            <ul className="p-8 space-y-2">
              <li className="mb-6">
                <Link
                  href="/"
                  className="py-2 rounded-md shadow-sm block px-3 text-gray-600 hover:text-gray-800 transition bg-white"
                >
                  Home
                </Link>
              </li>
              <li className="px-3 py-2 uppercase text-xs text-gray-800 font-semibold">
                Posts
              </li>
              {data?.postCollection?.edges.map(({ node }: any) => (
                <li key={node.id}>
                  <Link
                    href={`/posts/${node.slug}`}
                    className="py-2 rounded-md shadow-sm block px-3 text-gray-600 hover:text-gray-800 transition bg-white"
                  >
                    {node.title}
                  </Link>
                </li>
              ))}
            </ul>
          </nav>
          <main className="flex-1 py-6 md:py-24">
            <div className="max-w-3xl mx-auto">
              <div className="prose max-w-none">{children}</div>
            </div>
          </main>
        </div>
      </body>
    </html>
  )
}

export default RootLayout
