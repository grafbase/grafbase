# Grafbase ⨯ Clerk ⨯ Next.js

This examples shows how to connect Clerk as your Identity Provider with your Grafbase project &mdash; [Read the guide](https://grafbase.com/guides/using-clerk-as-your-identity-provider-with-grafbase)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/nextjs-clerk-hacker-news grafnews-with-clerk` to clone this example
2. Change directory into the new folder `cd grafnews-with-clerk`
3. Run `cp .env.example .env` to copy the example `.env.example` file to `.env` in the root of your project
4. Run `cp grafbase/.env.example grafbase/.env` to copy the example `.env.example` file to `.env` in the `grafbase` folder
5. Open `.env` in your code editor, and provide your Grafbase API endpoint and [Clerk frontend and backend API keys](https://dashboard.clerk.dev/last-active?path=api-keys)
6. Open `grafbase/.env` in your code editor, and provide your Clerk issuer URL
7. Run `npm install`, or `yarn install` to install dependencies
8. Run `npx grafbase dev` to start local dev server with your schema
9. Run `npm run dev`, or `yarn dev` (in a new terminal)
10. Visit [http://localhost:3000](http://localhost:3000)

## Missing
-[ ] Pagination of items, users and posts.
-[ ] Better typing by using fragments and remove the NonNullable thing.
-[ ] About page.


## Pending issues and technical debt

### General

#### Auth, no way to have both, anonymous and authenticated

I want the page to be fully visible to anonymous visitors but I can't with the current auth, as it is one or the other.


#### Auth, entities cannot be restricted to their owner

Any entity can be deleted by anyone once they access to it. Roles which are defined on Clerk cannot be automated. You must
assign them manually.

#### Deletion

As all links inside an entity get deleted when using a delete mutation, you cannot delete anything without also deleting the linked user.

### Viewer

#### Storing users manually

There is no way to hook Clerk and your Grafbase backend to automatically insert users on sign up, so you must hack it, and
create them manually on the login callback from Clerk.

#### Query your user

There is no way to query or store the user by certain prop, example, clerkId or username, or email, so I have to query
all the users and find the one that matches (email or username) client-side to hydrate the viewer.

#### No createdAt/updatedAt

I have to manually store the ms in which the user was created.

### Posts

#### No ordering by certain field (votes)
...

#### Not able to increment, decrement or aggregates on votes count
...

#### How to tell if a user has already voted a post?

I should be able to query Votes entity by postId and userId

### Bugs

#### Update mutation doesn't work a second time

Seems like update mutation only works once, you can check it out by upvoting/downvoting posts.
