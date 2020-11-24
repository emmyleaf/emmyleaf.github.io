import type { AppProps } from 'next/app'
import { FunctionComponent } from 'react'
import '../styles/index.css'

const App: FunctionComponent<AppProps> = ({ Component, pageProps }) => {
  return <Component {...pageProps} />
}

export default App
