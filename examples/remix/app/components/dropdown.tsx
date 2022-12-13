import { Menu, Transition } from '@headlessui/react'
import type { ReactNode } from 'react'
import { Fragment } from 'react'

export const Dropdown = ({
  trigger = 'Menu',
  children
}: Record<'trigger' | 'children', ReactNode>) => {
  return (
    <Menu>
      <Menu.Button>{trigger}</Menu.Button>
      <Transition
        as={Fragment}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <Menu.Items className="absolute right-0 w-56 px-1 py-1 mt-2 origin-top-right bg-white border border-gray-200 rounded-md shadow-lg dark:border-gray-700 dark:bg-zinc-900 ring-1 ring-black ring-opacity-5 focus:outline-none">
          {children}
        </Menu.Items>
      </Transition>
    </Menu>
  )
}

export const DropdownItem = (props: { children: ReactNode }) => (
  <Menu.Item {...props} />
)
