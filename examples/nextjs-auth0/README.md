# Grafbase ⨯ Auth0 ⨯ Next.js

This examples shows how to connect Auth0 as your Identity Provider with your Grafbase project &mdash; [Read the guide](https://grafbase.com/guides/using-auth0-as-your-identity-provider-with-grafbase)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/nextjs-auth0 grafbase-with-nextjs-auth0` to clone this example
2. Change directory into the new folder `cd grafbase-with-nextjs-auth0`
3. Run `cp .env.example .env`
4. Run `cp grafbase/.env.example grafbase/.env`
5. Open `.env` in your code editor, and provide your Grafbase API endpoint and [Auth0 Tenant Info](https://manage.auth0.com/dashboard)
6. Open `grafbase/.env` in your code editor, and provide the url of your Auth0 tenant domain
7. Run `npm install`, or `yarn install` to install dependencies
8. Run `npx grafbase dev` to start local dev server with your schema
9. Run `npm run dev`, or `yarn dev` (in a new terminal)
10. Visit [http://localhost:3000](http://localhost:3000)
