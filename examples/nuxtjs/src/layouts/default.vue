<template>
  <div>
    <Head>
      <Title>Nuxt.js - Todo Example - Grafbase</Title>
      <Meta
        name="description"
        content="Todo Example leveraging the Grafbase platform"
      />
      <Link rel="shortcut icon" href="/favicon.png" />
    </Head>

    <Body class="text-black dark:text-white">
      <NavMenu />
      <main
        class="relative h-[calc(100vh_-_64px)] flex pl-6 pt-6 pr-12 overflow-x-auto dark:bg-zinc-900 transition-colors duration-300"
      >
        <div
          className="fixed left-0 w-6 h-full bg-gradient-to-l from-transparent to-white dark:to-zinc-900"
        />
        <div
          className="fixed right-0 w-12 h-full bg-gradient-to-r from-transparent to-white dark:to-zinc-900"
        />
        <NuxtPage />
      </main>
    </Body>
  </div>
</template>
