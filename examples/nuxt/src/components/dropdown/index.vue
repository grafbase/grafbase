<script setup lang="ts">
import { Menu, MenuButton, MenuItems } from '@headlessui/vue'
</script>

<template>
  <Menu>
    <MenuButton>
      <slot name="trigger">Menu</slot>
    </MenuButton>
    <transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-out"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <MenuItems
        class="absolute right-0 w-56 px-1 py-1 mt-2 origin-top-right bg-white border border-gray-200 rounded-md shadow-lg dark:border-gray-700 dark:bg-zinc-900 ring-1 ring-black ring-opacity-5 focus:outline-none"
      >
        <slot></slot>
      </MenuItems>
    </transition>
  </Menu>
</template>
