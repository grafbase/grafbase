import React from 'react'
import { Playground } from '@grafbase/graphiql'
import './App.css'

function App() {
  return (
    <div className="App">
      <Playground
        logo={<></>}
        endpoint={(window as any).GRAPHQL_URL}
        defaultHeaders={'{ "x-api-key": "" }'}
      ></Playground>
    </div>
  )
}

export default App
