import { useAuth } from '@clerk/nextjs'
import Img from 'components/img'
import formatDistanceToNow from 'date-fns/formatDistanceToNow'
import useViewer from 'hooks/use-viewer'
import Head from 'next/head'

const UserIdPage = () => {
  const { signOut } = useAuth()
  const { viewer, loading } = useViewer()

  if (loading) {
    return (
      <div className="flex">
        <div className="animate-pulse bg-gray-200 w-14 h-14" />
        <div className="animate-pulse ml-4 bg-gray-200 h-14 w-[250px]" />
      </div>
    )
  }

  if (!loading && !viewer) {
    return (
      <div className="bg-red-500 min-h-24 w-full flex flex-col space-y-6 items-center justify-center py-6">
        <div className="text-lg text-white">
          Something went wrong in the API. Please, disconnect and login again.
        </div>
        <div>
          <button
            onClick={() => {
              signOut().then(() => localStorage.removeItem('hasLoggedIn'))
            }}
            className="px-2 py-1 bg-red-900 text-white text-xl"
          >
            Log Out
          </button>
        </div>
      </div>
    )
  }

  return (
    <div>
      <Head>
        <title>{viewer?.name || 'User'} | Grafnews</title>
      </Head>
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-4">
          <Img alt="User image" src={viewer?.imageUrl} className="w-14 h-14" />
          <h1 className="text-5xl font-bold">{viewer?.name}</h1>
        </div>
        <button
          onClick={() => signOut()}
          className="px-2 py-1 bg-red-700 text-white text-xl"
        >
          Logout
        </button>
      </div>
      <div className="border-b-4 mt-6 max-w-sm border-black" />
      <p className="text-xl mt-4 text-gray-600">
        {viewer?.email} | Joined{' '}
        <time>
          {!!viewer?.createdAt &&
            formatDistanceToNow(Date.parse(viewer?.createdAt), {
              addSuffix: true
            })}
        </time>
      </p>
    </div>
  )
}

export default UserIdPage
