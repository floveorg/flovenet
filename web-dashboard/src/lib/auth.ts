const TOKEN_KEY = 'flovenet_token'
const PROFILE_KEY = 'flovenet_profile'

export interface Profile {
  peerId: string
  displayName: string
  bio?: string | null
  avatarCid?: string | null
  followerCount: number
  followingCount: number
  postCount: number
  reputation?: number
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function getProfile(): Profile | null {
  const raw = localStorage.getItem(PROFILE_KEY)
  return raw ? JSON.parse(raw) : null
}

export function saveAuth(token: string, profile: Profile) {
  localStorage.setItem(TOKEN_KEY, token)
  localStorage.setItem(PROFILE_KEY, JSON.stringify(profile))
}

export function clearAuth() {
  localStorage.removeItem(TOKEN_KEY)
  localStorage.removeItem(PROFILE_KEY)
}

export function authHeaders(): Record<string, string> {
  const token = getToken()
  return token ? { Authorization: `Bearer ${token}` } : {}
}
