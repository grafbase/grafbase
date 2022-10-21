import { useAuth, useSession } from '@clerk/nextjs'
import LogoAnimated from 'components/logo-animated'
import Link from 'next/link'
import { useRouter } from 'next/router'
import { PropsWithChildren, SVGProps } from 'react'

const navigation = [
  { name: 'Feed', href: '/' },
  { name: 'Users', href: '/users' },
  { name: 'Submit', href: '/item/submit' }
  // { name: 'About', href: '/about' }
]

const footer = [
  {
    name: 'Twitter',
    href: 'https://twitter.com/grafbase',
    icon: (props: SVGProps<SVGSVGElement>) => (
      <svg fill="currentColor" viewBox="0 0 24 24" {...props}>
        <path d="M8.29 20.251c7.547 0 11.675-6.253 11.675-11.675 0-.178 0-.355-.012-.53A8.348 8.348 0 0022 5.92a8.19 8.19 0 01-2.357.646 4.118 4.118 0 001.804-2.27 8.224 8.224 0 01-2.605.996 4.107 4.107 0 00-6.993 3.743 11.65 11.65 0 01-8.457-4.287 4.106 4.106 0 001.27 5.477A4.072 4.072 0 012.8 9.713v.052a4.105 4.105 0 003.292 4.022 4.095 4.095 0 01-1.853.07 4.108 4.108 0 003.834 2.85A8.233 8.233 0 012 18.407a11.616 11.616 0 006.29 1.84" />
      </svg>
    )
  },
  {
    name: 'GitHub',
    href: 'https://github.com/grafbase',
    icon: (props: SVGProps<SVGSVGElement>) => (
      <svg fill="currentColor" viewBox="0 0 24 24" {...props}>
        <path
          fillRule="evenodd"
          d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
          clipRule="evenodd"
        />
      </svg>
    )
  },
  {
    name: 'Discord',
    href: 'https://discord.com/invite/grafbase',
    icon: (props: SVGProps<SVGSVGElement>) => (
      <svg
        fill="currentColor"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 640 512"
        {...props}
      >
        <path d="M524.531,69.836a1.5,1.5,0,0,0-.764-.7A485.065,485.065,0,0,0,404.081,32.03a1.816,1.816,0,0,0-1.923.91,337.461,337.461,0,0,0-14.9,30.6,447.848,447.848,0,0,0-134.426,0,309.541,309.541,0,0,0-15.135-30.6,1.89,1.89,0,0,0-1.924-.91A483.689,483.689,0,0,0,116.085,69.137a1.712,1.712,0,0,0-.788.676C39.068,183.651,18.186,294.69,28.43,404.354a2.016,2.016,0,0,0,.765,1.375A487.666,487.666,0,0,0,176.02,479.918a1.9,1.9,0,0,0,2.063-.676A348.2,348.2,0,0,0,208.12,430.4a1.86,1.86,0,0,0-1.019-2.588,321.173,321.173,0,0,1-45.868-21.853,1.885,1.885,0,0,1-.185-3.126c3.082-2.309,6.166-4.711,9.109-7.137a1.819,1.819,0,0,1,1.9-.256c96.229,43.917,200.41,43.917,295.5,0a1.812,1.812,0,0,1,1.924.233c2.944,2.426,6.027,4.851,9.132,7.16a1.884,1.884,0,0,1-.162,3.126,301.407,301.407,0,0,1-45.89,21.83,1.875,1.875,0,0,0-1,2.611,391.055,391.055,0,0,0,30.014,48.815,1.864,1.864,0,0,0,2.063.7A486.048,486.048,0,0,0,610.7,405.729a1.882,1.882,0,0,0,.765-1.352C623.729,277.594,590.933,167.465,524.531,69.836ZM222.491,337.58c-28.972,0-52.844-26.587-52.844-59.239S193.056,219.1,222.491,219.1c29.665,0,53.306,26.82,52.843,59.239C275.334,310.993,251.924,337.58,222.491,337.58Zm195.38,0c-28.971,0-52.843-26.587-52.843-59.239S388.437,219.1,417.871,219.1c29.667,0,53.307,26.82,52.844,59.239C470.715,310.993,447.538,337.58,417.871,337.58Z" />
      </svg>
    )
  }
]

const Layout = ({ children }: PropsWithChildren) => {
  const { asPath } = useRouter()
  const { isLoaded, isSignedIn, userId } = useAuth()
  const { session } = useSession()

  return (
    <div>
      <header className="bg-black">
        <nav
          className="mx-auto max-w-screen-md px-4 sm:px-6 lg:px-8"
          aria-label="Top"
        >
          <div className="flex w-full items-center justify-between border-b py-4 lg:border-none">
            <div className="flex items-center">
              <Link href="/" passHref>
                <a className="flex items-center space-x-3">
                  <LogoAnimated className="text-white w-6 h-6" />
                  <span className="text-white font-semibold text-xl">
                    Grafnews
                  </span>
                </a>
              </Link>
              <div className="ml-10 hidden space-x-8 lg:block">
                {navigation.map(({ href, name }) => (
                  <Link key={name} href={href} passHref>
                    <a
                      className={
                        'text-base text-white hover:text-indigo-50 ' +
                        (asPath === href && 'font-bold')
                      }
                    >
                      {name}
                    </a>
                  </Link>
                ))}
              </div>
            </div>
            {isSignedIn ? (
              <div className="ml-10 space-x-4">
                <Link
                  href={{ pathname: '/user/[id]', query: { id: userId } }}
                  passHref
                >
                  <a className="inline-block rounded border border-transparent text-base font-bold text-white hover:bg-opacity-75">
                    {session?.user?.username}
                  </a>
                </Link>
              </div>
            ) : (
              <div className="ml-10 space-x-4">
                <Link href="/login" passHref>
                  <a className="inline-block rounded border border-transparent text-base font-bold text-white hover:bg-opacity-75">
                    {isLoaded ? 'Join' : '...'}
                  </a>
                </Link>
              </div>
            )}
          </div>
          <div className="flex flex-wrap justify-center space-x-6 py-4 lg:hidden">
            {navigation.map(({ href, name }) => (
              <Link key={name} href={href} passHref>
                <a className="text-base  text-white hover:text-indigo-50">
                  {name}
                </a>
              </Link>
            ))}
          </div>
        </nav>
      </header>
      <main className="h-full min-h-screen max-w-screen-md mx-auto my-8 px-4 sm:px-6 ">
        {children}
      </main>
      <footer className="bg-white border-t">
        <div className="mx-auto max-w-7xl py-12 px-4 sm:px-6 md:flex md:items-center md:justify-between lg:px-8">
          <div className="flex justify-center space-x-6 md:order-2">
            {footer.map((item) => (
              <a
                key={item.name}
                href={item.href}
                className="text-gray-500 hover:text-gray-500"
              >
                <span className="sr-only">{item.name}</span>
                <item.icon className="h-6 w-6" aria-hidden="true" />
              </a>
            ))}
          </div>
          <div className="mt-8 md:order-1 md:mt-0">
            <p className="text-center text-base text-gray-700">
              &copy; Grafbase, Inc. All rights reserved.
            </p>
          </div>
        </div>
      </footer>
    </div>
  )
}

export default Layout
