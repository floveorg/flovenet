import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Client, Provider, cacheExchange, fetchExchange } from 'urql'
import App from './App'
import './index.css'

const client = new Client({
  url: '/graphql',
  exchanges: [cacheExchange, fetchExchange],
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Provider value={client}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </Provider>
  </React.StrictMode>,
)
