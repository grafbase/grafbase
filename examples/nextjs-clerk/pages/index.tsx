import { SignedIn, SignedOut, useAuth } from '@clerk/nextjs'
import type { NextPage } from 'next'
import Link from 'next/link'

const Home: NextPage = () => {
  const { signOut } = useAuth()

  return (
    <main>
      <SignedIn>
        <p>You have successfully signed in</p>
        <button onClick={() => signOut()}>Sign Out</button>
      </SignedIn>
      <SignedOut>
        <p>Sign up for an account to get started</p>
        <Link href="/sign-up">
          <button>Sign Up</button>
        </Link>
      </SignedOut>
    </main>
  )
}

export default Home
