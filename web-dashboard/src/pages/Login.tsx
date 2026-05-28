import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { graphqlRequest } from '../lib/graphql-client'
import { saveAuth } from '../lib/auth'
import { LOGIN } from '../graphql/queries'

export default function Login() {
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      const data = await graphqlRequest(LOGIN, { email, password })
      saveAuth(data.login.token, data.login.profile)
      navigate('/')
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Login failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="container" style={{ maxWidth: 420, marginTop: 80 }}>
      <div className="card">
        <h1 className="text-xl font-bold mb-4 text-center">Flovenet</h1>
        <p className="text-muted text-center text-sm mb-4">Sign in to your account</p>
        <form onSubmit={handleSubmit} className="flex-col gap-4" style={{ display: 'flex' }}>
          <input
            type="email"
            placeholder="Email"
            value={email}
            onChange={e => setEmail(e.target.value)}
            required
          />
          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={e => setPassword(e.target.value)}
            required
          />
          {error && <p style={{ color: 'var(--red)', fontSize: 13 }}>{error}</p>}
          <button type="submit" disabled={loading}>
            {loading ? 'Signing in...' : 'Sign In'}
          </button>
        </form>
        <p className="text-center text-sm mt-4 text-muted">
          Don't have an account? <Link to="/register">Register</Link>
        </p>
      </div>
    </div>
  )
}
