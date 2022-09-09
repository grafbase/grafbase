import { SignedIn, SignedOut } from '@clerk/nextjs'
import type { NextPage } from 'next'

const Home: NextPage = () => {
  return (
    <>
      <SignedIn>
        <p>You have successfully signed in</p>
      </SignedIn>
      <SignedOut>
        <p>Sign up for an account to get started</p>
      </SignedOut>
    </>
  )
}

export default Home
