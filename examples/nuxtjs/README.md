# Grafbase ⨯ Nuxt 3

[Join our Discord](https://discord.gg/grafbase)

[![Deploy to Grafbase](https://grafbase.com/button)](https://grafbase.com/new/configure?template=Todo&source=https%3A%2F%2Fgithub.com%2Fgrafbase%2Fgrafbase%2Ftree%2Fmain%2Ftemplates%2Ftodo)
</br>
[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/import/project?template=https://github.com/grafbase/grafbase/tree/main/examples/nextjs)

## Mandatory configuration

Rename your `.env.example` to `.env` and fill all the variables.

### GitHub

Create a GitHub OAuth application and make sure to set:

- Homepage URL: http://localhost:3000
- Authorization callback URL: http://localhost:3000/api/github/callback

Fill your `.env` with `GITHUB_CLIENT_ID` and` GITHUB_CLIENT_SECRET` variables.

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `src/pages/sign-in.vue`. The page auto-updates as you edit the file.

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

To learn more about Nuxt 3, take a look at the following resources:

- [Nuxt 3 Documentation](https://v3.nuxtjs.org/guide/concepts/introduction) - learn about Nuxt 3 features and API.

### Run on Codesandbox

[![Develop with CodeSandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/nuxtjs)
