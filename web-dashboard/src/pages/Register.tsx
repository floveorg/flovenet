import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { graphqlRequest } from '../lib/graphql-client'
import { saveAuth } from '../lib/auth'
import { REGISTER } from '../graphql/queries'

export default function Register() {
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      const data = await graphqlRequest(REGISTER, { email, password, displayName })
      saveAuth(data.register.token, data.register.profile)
      navigate('/')
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Registration failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="container" style={{ maxWidth: 420, marginTop: 80 }}>
      <div className="card">
        <h1 className="text-xl font-bold mb-4 text-center">Join Flovenet</h1>
        <p className="text-muted text-center text-sm mb-4">
          Create an account to start sharing compute
        </p>
        <form onSubmit={handleSubmit} className="flex-col gap-4" style={{ display: 'flex' }}>
          <input
            type="text"
            placeholder="Display Name"
            value={displayName}
            onChange={e => setDisplayName(e.target.value)}
            required
          />
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
            minLength={8}
          />
          {error && <p style={{ color: 'var(--red)', fontSize: 13 }}>{error}</p>}
          <button type="submit" disabled={loading}>
            {loading ? 'Creating account...' : 'Create Account'}
          </button>
        </form>
        <p className="text-center text-sm mt-4 text-muted">
          Already have an account? <Link to="/login">Sign in</Link>
        </p>
      </div>
    </div>
  )
}
