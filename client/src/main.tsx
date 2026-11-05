import React from 'react'
import ReactDOM from 'react-dom/client'
import { App } from './App'
import './styles.css'

import { TopDownView } from './TopDownView'
const scene = new URLSearchParams(location.search).get('scene')
const Root = scene === 'radar' ? App : () => <TopDownView gameplay={!(import.meta.env.DEV && scene === 'top-down')} />

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><React.Suspense fallback={null}><Root /></React.Suspense></React.StrictMode>
)
