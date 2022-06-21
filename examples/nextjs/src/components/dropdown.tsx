import { Menu, Transition } from "@headlessui/react";
import { Fragment, ReactNode } from "react";
import * as React from "react";

type Option = {
  name: string;
  icon: (props: React.ComponentProps<"svg">) => JSX.Element;
  onClick: () => void;
};

type Props = {
  children: ReactNode;
  options: Option[];
};

const Dropdown = ({ children, options }: Props) => {
  return (
    <Menu>
      <Menu.Button>{children}</Menu.Button>
      <Transition
        as={Fragment}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <Menu.Items className="absolute right-0 mt-2 w-56 origin-top-right border border-gray-200 dark:border-gray-700 divide-y divide-gray-100 rounded-md bg-white dark:bg-zinc-900 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
          <div className="px-1 py-1 ">
            {options.map(({ onClick, name, ...props }, index) => (
              <Menu.Item key={index}>
                {() => (
                  <button
                    onClick={onClick}
                    className="group flex w-full items-center rounded-md px-2 py-2 text-sm hover:bg-gray-200 hover:dark:bg-zinc-700"
                  >
                    <props.icon className="mr-3 h-5 w-5" aria-hidden="true" />
                    {name}
                  </button>
                )}
              </Menu.Item>
            ))}
          </div>
        </Menu.Items>
      </Transition>
    </Menu>
  );
};

export default Dropdown;
