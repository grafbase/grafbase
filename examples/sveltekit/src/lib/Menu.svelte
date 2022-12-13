<script lang="ts">
  import {
    Menu,
    MenuButton,
    MenuItems,
    MenuItem,
    Transition
  } from '@rgossiaux/svelte-headlessui'

  type Option = {
    name: string
    onClick: () => void
  }

  export let options: Option[]
</script>

<div class="relative inline-block text-left">
  <Menu>
    <span class="rounded-md shadow-sm">
      <MenuButton>
        <slot />
      </MenuButton>
    </span>

    <Transition
      enter="transition duration-250 ease-out"
      enterFrom="transform scale-95 opacity-0"
      enterTo="transform scale-100 opacity-100"
      leave="transition duration-100 ease-out"
      leaveFrom="transform scale-100 opacity-100"
      leaveTo="transform scale-95 opacity-0"
    >
      <MenuItems
        class="absolute right-0 w-56 mt-2 origin-top-right bg-white border border-gray-200 divide-y divide-gray-100 rounded-md shadow-lg outline-none"
      >
        <div class="py-1">
          {#each options as { name, onClick }}
            <MenuItem
              class="flex justify-between w-full px-4 py-2 text-sm leading-5 text-left hover:bg-gray-100 cursor-pointer"
              on:click={onClick}
            >
              {name}
            </MenuItem>
          {/each}
        </div>
      </MenuItems>
    </Transition>
  </Menu>
</div>
