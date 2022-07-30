<script setup lang="ts">
import { MenuItem } from "@headlessui/vue";
</script>

<template>
  <MenuItem
    class="flex items-center w-full px-2 py-2 text-sm rounded-md group hover:bg-emerald-200 hover:dark:bg-emerald-700"
    as="button"
  >
    <slot></slot>
  </MenuItem>
</template>
