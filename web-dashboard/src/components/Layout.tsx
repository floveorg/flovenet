import { Link, Outlet, useNavigate } from 'react-router-dom'
import { clearAuth, getProfile } from '../lib/auth'

export default function Layout() {
  const navigate = useNavigate()
  const profile = getProfile()

  const handleLogout = () => {
    clearAuth()
    navigate('/login')
  }

  return (
    <div>
      <nav style={{
        background: 'var(--bg-card)',
        borderBottom: '1px solid var(--border)',
        padding: '12px 0',
      }}>
        <div className="container flex items-center justify-between">
          <div className="flex items-center gap-6">
            <Link to="/" style={{ fontWeight: 700, fontSize: 18, color: 'var(--text)' }}>
              Flovenet
            </Link>
            <Link to="/">Dashboard</Link>
            <Link to="/feed">Feed</Link>
            <Link to="/network">Network</Link>
          </div>
          <div className="flex items-center gap-4">
            <Link to="/profile" style={{ color: 'var(--text)' }}>
              {profile?.displayName || 'Profile'}
            </Link>
            <button onClick={handleLogout} style={{ background: 'transparent', border: '1px solid var(--border)', color: 'var(--text-muted)' }}>
              Logout
            </button>
          </div>
        </div>
      </nav>
      <main className="container" style={{ paddingTop: 32, paddingBottom: 32 }}>
        <Outlet />
      </main>
    </div>
  )
}
