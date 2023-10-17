# cli-app

A Vite-powered full page version of Pathfinder, for use in the CLI.

Running in dev and preview modes require use of the `.env.development.local` file.

To get started, copy the example file then fill out the two variables with your endpoint and apiKey.

```
cp .env.development.local.example .env.development.local
```

## Dev

```
pnpm cli-app:dev
```

## Build for CLI usage

The `vite.config.ts` for this app is configured to set the base path for generated assets to the S3 bucket where we host the files.

```
pnpm cli-app:build
```

The generated `dist` folder needs to be uploaded to the S3 bucket where we're hosting. Currently handled by Hugo, we'll need to fix this up so it's automatic as soon as possible.

## Preview

You can preview the build by running the below command. Because of the base path config, this will load assets remotely and use your local env file for endpoint information.

```
pnpm cli-app:preview
```
