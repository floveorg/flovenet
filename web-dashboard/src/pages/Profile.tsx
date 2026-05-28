import { useEffect, useState } from 'react'
import { getProfile } from '../lib/auth'
import { graphqlRequest } from '../lib/graphql-client'
import { GET_PROFILE, SEARCH_PROFILES, FOLLOW, UNFOLLOW, GET_FOLLOWING, GET_FOLLOWERS } from '../graphql/queries'

interface ProfileData {
  peerId: string
  displayName: string
  bio?: string | null
  followerCount: number
  followingCount: number
  postCount: number
  reputation?: number
}

export default function Profile() {
  const myProfile = getProfile()
  const [profile, setProfile] = useState<ProfileData | null>(null)
  const [following, setFollowing] = useState<ProfileData[]>([])
  const [followers, setFollowers] = useState<ProfileData[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<ProfileData[]>([])

  useEffect(() => {
    if (!myProfile?.peerId) return
    graphqlRequest(GET_PROFILE, { peerId: myProfile.peerId }).then(d => {
      setProfile(d.profile)
    })
    graphqlRequest(GET_FOLLOWING, { peerId: myProfile.peerId }).then(d => {
      setFollowing(d.following || [])
    })
    graphqlRequest(GET_FOLLOWERS, { peerId: myProfile.peerId }).then(d => {
      setFollowers(d.followers || [])
    })
  }, [myProfile?.peerId])

  const handleSearch = async () => {
    if (!searchQuery.trim()) return
    const data = await graphqlRequest(SEARCH_PROFILES, { query: searchQuery })
    setSearchResults(data.searchProfiles || [])
  }

  const handleFollow = async (peerId: string) => {
    await graphqlRequest(FOLLOW, { peerId })
    setFollowing(prev => [...prev, { peerId, displayName: '', followerCount: 0, followingCount: 0, postCount: 0 }])
  }

  const handleUnfollow = async (peerId: string) => {
    await graphqlRequest(UNFOLLOW, { peerId })
    setFollowing(prev => prev.filter(p => p.peerId !== peerId))
  }

  return (
    <div className="flex-col gap-6" style={{ display: 'flex' }}>
      <h1 className="text-xl font-bold">Profile</h1>

      <div className="card">
        <div className="flex-col gap-2 text-sm" style={{ display: 'flex' }}>
          <Row label="Peer ID" value={profile?.peerId} />
          <Row label="Display Name" value={profile?.displayName} />
          <Row label="Bio" value={profile?.bio || 'No bio'} />
          <Row label="Reputation" value={profile?.reputation?.toFixed(2)} />
          <Row label="Followers" value={profile?.followerCount?.toString()} />
          <Row label="Following" value={profile?.followingCount?.toString()} />
          <Row label="Posts" value={profile?.postCount?.toString()} />
        </div>
      </div>

      <div className="card">
        <h2 className="font-bold mb-4">Find Users</h2>
        <div className="flex gap-2">
          <input
            placeholder="Search by display name or peer ID"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleSearch()}
          />
          <button onClick={handleSearch}>Search</button>
        </div>
        {searchResults.length > 0 && (
          <div className="flex-col gap-2 mt-4" style={{ display: 'flex' }}>
            {searchResults.map(u => (
              <div key={u.peerId} className="flex items-center justify-between card" style={{ padding: 12 }}>
                <div>
                  <p className="font-bold">{u.displayName}</p>
                  <p className="text-muted text-sm">{u.peerId.slice(0, 16)}...</p>
                </div>
                {u.peerId !== myProfile?.peerId && (
                  following.find(f => f.peerId === u.peerId)
                    ? <button onClick={() => handleUnfollow(u.peerId)} style={{ background: 'var(--red)' }}>Unfollow</button>
                    : <button onClick={() => handleFollow(u.peerId)}>Follow</button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="grid grid-2 gap-4">
        <div className="card">
          <h2 className="font-bold mb-4">Following ({following.length})</h2>
          <div className="flex-col gap-2" style={{ display: 'flex' }}>
            {following.map(f => (
              <div key={f.peerId} className="flex items-center justify-between">
                <span className="text-sm">{f.displayName || f.peerId.slice(0, 16)}</span>
                <button onClick={() => handleUnfollow(f.peerId)} style={{ background: 'var(--red)', padding: '4px 12px', fontSize: 12 }}>
                  Unfollow
                </button>
              </div>
            ))}
            {following.length === 0 && <p className="text-muted text-sm">Not following anyone yet</p>}
          </div>
        </div>

        <div className="card">
          <h2 className="font-bold mb-4">Followers ({followers.length})</h2>
          <div className="flex-col gap-2" style={{ display: 'flex' }}>
            {followers.map(f => (
              <div key={f.peerId} className="text-sm">
                {f.displayName || f.peerId.slice(0, 16)}
              </div>
            ))}
            {followers.length === 0 && <p className="text-muted text-sm">No followers yet</p>}
          </div>
        </div>
      </div>
    </div>
  )
}

function Row({ label, value }: { label: string; value?: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-muted">{label}</span>
      <span style={{ maxWidth: 300, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {value || '-'}
      </span>
    </div>
  )
}
