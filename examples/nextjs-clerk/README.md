# Grafbase ⨯ Clerk ⨯ Next.js

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/nextjs-clerk grafbase-with-nextjs-clerk` to clone this example
1. Change directory into the new folder `cd grafbase-with-nextjs-clerk`
1. Run `cp .env.example .env` to copy the example `.env.example` file to `.env`
1. Open `.env` in your code editor, and provide your Grafbase API endpoint and Clerk frontend and backend API keys, which you can copy from [here](https://dashboard.clerk.dev/last-active?path=api-keys)
1. Open `grafbase/schema.graphql` and replace `{{ env.NEXT_PUBLIC_CLERK_FRONTEND_API }}` with Clerk frontend API key \
   **TODO**: remove this step, when new version of CLI is released
1. Run `npm install`, or `yarn install` to install dependencies
1. Run `npx grafbase dev` to start local dev server with your schema
1. Run `npm run dev`, or `yarn dev` (in a new terminal)
1. Visit http://localhost:3000
