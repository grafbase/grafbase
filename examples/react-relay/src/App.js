import { RelayEnvironmentProvider } from 'react-relay/hooks'
import { BrowserRouter, Routes, Route } from 'react-router-dom'

import RelayEnvironment from './environment'
import IndexPage from './IndexPage'

function App() {
  return (
    <RelayEnvironmentProvider environment={RelayEnvironment}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<IndexPage />} />
        </Routes>
      </BrowserRouter>
    </RelayEnvironmentProvider>
  )
}

export default App
