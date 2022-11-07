// @refresh reload
import { Suspense } from 'solid-js'
import { Body, ErrorBoundary, FileRoutes, Head, Html, Link, Meta, Routes, Scripts, Title } from 'solid-start'
import './root.css'

export default function Root() {
  return (
    <Html lang='en'>
      <Head>
        <Meta charset='utf-8' />
        <Meta name='viewport' content='width=device-width, initial-scale=1' />
        <Link rel='shortcut icon' type='image/png' href='/favicon.png' />
        <Title>SolidStart - Todo Example - Grafbase</Title>
        <Meta name='description' content='Todo Example leveraging the Grafbase platform' />
      </Head>
      <Body>
        <Suspense>
          <ErrorBoundary>
            <Routes>
              <FileRoutes />
            </Routes>
          </ErrorBoundary>
        </Suspense>
        <Scripts />
      </Body>
    </Html>
  )
}
