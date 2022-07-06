<script setup lang="ts">
const theme = useColorMode()
const toggleTheme = () => {
  theme.preference = theme.value === 'dark' ? 'light' : 'dark'
}
</script>

<template>
  <ColorScheme>
    <span class="h-8">
      <label v-show="!theme.unknown" htmlFor="theme-switcher" class="relative inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          id="theme-switcher"
          :checked="theme.value === 'dark'"
          @change="toggleTheme"
          class="sr-only peer"
        />

        <IconRadixIconsSun class="absolute z-10 w-4 h-4 left-2 dark:text-gray-400" />
        <IconRadixIconsMoon class="absolute z-10 w-4 h-4 text-gray-400 right-2 dark:text-white" />
        <div
          class="w-14 h-8 bg-[#38361F] peer-focus:outline-none peer-focus:ring-0 rounded-full dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-none after:absolute after:top-[4px] after:left-[4px] after:bg-yellow-400 after:dark:bg-blue-500 after:rounded-full after:h-6 after:w-6 after:transition-all dark:border-gray-600 peer-checked:bg-[#303853] peer-checked:dark:bg-blue-200"
        ></div>
      </label>
    </span>
  </ColorScheme>
</template>
