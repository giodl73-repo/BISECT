/**
 * Main application component with routing.
 */
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { ErrorBoundary } from './components/ErrorBoundary'
import { Layout } from './components/Layout'
import { RunList } from './pages/RunList'
import { RunDetail } from './pages/RunDetail'
import { CreateRun } from './pages/CreateRun'
import { Districts } from './pages/Districts'
import { About } from './pages/About'

function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<RunList />} />
            <Route path="/runs/:id" element={<RunDetail />} />
            <Route path="/runs/:id/districts" element={<Districts />} />
            <Route path="/create" element={<CreateRun />} />
            <Route path="/about" element={<About />} />
          </Routes>
        </Layout>
      </BrowserRouter>
    </ErrorBoundary>
  )
}

export default App
