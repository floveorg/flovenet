import { Routes, Route, Navigate } from 'react-router-dom'
import Layout from './components/Layout'
import Login from './pages/Login'
import Register from './pages/Register'
import Dashboard from './pages/Dashboard'
import Profile from './pages/Profile'
import Feed from './pages/Feed'
import Network from './pages/Network'

function App() {
  const token = localStorage.getItem('flovenet_token')

  return (
    <Routes>
      <Route path="/login" element={token ? <Navigate to="/" /> : <Login />} />
      <Route path="/register" element={token ? <Navigate to="/" /> : <Register />} />
      <Route element={<Layout />}>
        <Route path="/" element={token ? <Dashboard /> : <Navigate to="/login" />} />
        <Route path="/profile" element={token ? <Profile /> : <Navigate to="/login" />} />
        <Route path="/feed" element={token ? <Feed /> : <Navigate to="/login" />} />
        <Route path="/network" element={token ? <Network /> : <Navigate to="/login" />} />
      </Route>
    </Routes>
  )
}

export default App
