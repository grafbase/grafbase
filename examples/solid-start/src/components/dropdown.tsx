import { JSXElement, ParentComponent } from 'solid-js'

export const Dropdown: ParentComponent<{ trigger: JSXElement }> = ({
  trigger = 'Menu',
  children
}) => {
  return (
    <div class="dropdown dropdown-end">
      <label tabindex="0" class="cursor-pointer">
        {trigger}
      </label>
      <ul
        tabindex="0"
        class="dropdown-content menu w-52 px-1 py-1 mt-2 origin-top-right bg-white border border-gray-200 rounded-md shadow-lg dark:border-gray-700 dark:bg-zinc-900 ring-1 ring-black ring-opacity-5 focus:outline-none"
      >
        {children}
      </ul>
    </div>
  )
}

export const DropdownItem: ParentComponent = (props) => <li {...props} />
