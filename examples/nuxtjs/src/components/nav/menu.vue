<script setup lang="ts">
const user = await useGithubUser()

const logout = async () => {
  githubLogout()
  await navigateTo('/sign-in')
}
</script>

<template>
  <nav class="flex flex-wrap items-center justify-between p-4 bg-zinc-100 dark:bg-zinc-800">
    <LogoGrafbase />
    <div class="flex items-center space-x-4">
      <button
        v-if="user"
        @click="logout"
        class="px-2 py-1 text-sm border border-gray-400 rounded-lg dark:border-gray-600"
      >
        Sign Out
      </button>
      <ThemeSwitcher />
    </div>
  </nav>
</template>
