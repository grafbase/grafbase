<template>
  <div class="flex justify-center flex-1 py-36 item-center animate-show">
    <div
      class="flex flex-col items-center p-6 space-y-4 border border-gray-200 rounded-lg shadow-lg h-fit dark:border-gray-700"
    >
      <h2 class="text-xl font-semibold">Nuxt.js - Todo Example</h2>
      <button
        @click="githubLogin"
        class="flex items-center px-5 py-3 space-x-2 text-white bg-black rounded-lg dark:bg-zinc-700 hover:bg-gray-900"
      >
        <IconRadixIconsGithubLogo />
        <span>Sign In with GitHub</span>
      </button>
    </div>
  </div>
</template>
