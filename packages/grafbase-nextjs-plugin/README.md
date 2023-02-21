# @grafbase/nextjs-plugin

A Next.js "plugin" that automatically starts the Grafbase CLI.

## Usage

It's easy as 123!

### Step 1

Install `grafbase` as a `devDependency`:

```bash
npm install -D grafbase @grafbase/nextjs-plugin
```

### Step 2

Add the following to `.env`:

```bash
GRAFBASE_API_URL=http://localhost:4000/graphql
# OR
# NEXT_PUBLIC_GRAFBASE_API_URL=http://localhost:4000/graphql
```

### Step 3

Then inside `next.config.js` import `withGrafbase`, and wrap your exported config:

```ts
/** @type {import('next').NextConfig} */

const { withGrafbase } = require('@grafbase/nextjs-plugin')

const nextConfig = () =>
  withGrafbase({
    reactStrictMode: true,
    swcMinify: true
  })

module.exports = nextConfig
```

Finally run your Next.js app! The Grafbase CLI will be running with your backend.

## Notes

If there is no environment variable the [Grafbase CLI](https://grafbase.com/cli) will not start.
