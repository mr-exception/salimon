import React from 'react'
import ReactDOM from 'react-dom/client'
import { App } from './App'
import './styles.css'

const Root = import.meta.env.DEV && new URLSearchParams(location.search).get('scene') === 'top-down'
  ? React.lazy(() => import('./TopDownView').then(module => ({ default: module.TopDownView })))
  : App

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><React.Suspense fallback={null}><Root /></React.Suspense></React.StrictMode>
)
