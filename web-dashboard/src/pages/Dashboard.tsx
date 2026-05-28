import { useEffect, useState } from 'react'
import { getProfile, type Profile } from '../lib/auth'

interface NodeInfo {
  platform: string
  peerId: string
  cpuCores: number
  ramGb: number
  diskGb: number
  uptimeSecs: number
}

export default function Dashboard() {
  const profile = getProfile()
  const [nodeInfo, setNodeInfo] = useState<NodeInfo | null>(null)

  useEffect(() => {
    const cached = sessionStorage.getItem('flovenet_node_info')
    if (cached) {
      setNodeInfo(JSON.parse(cached))
      return
    }
    // Try to fetch node info from local metrics endpoint
    fetch('/health')
      .then(r => r.text())
      .then(async () => {
        // If we can reach the gateway, fetch resource info via GraphQL
        const info: NodeInfo = {
          platform: navigator.platform,
          peerId: profile?.peerId || 'unknown',
          cpuCores: navigator.hardwareConcurrency || 0,
          ramGb: (performance as any)?.memory?.jsHeapSizeLimit
            ? (performance as any).memory.jsHeapSizeLimit / 1024 / 1024 / 1024
            : 0,
          diskGb: 0,
          uptimeSecs: Math.floor(performance.now() / 1000),
        }
        sessionStorage.setItem('flovenet_node_info', JSON.stringify(info))
        setNodeInfo(info)
      })
      .catch(() => {
        setNodeInfo(null)
      })
  }, [profile])

  return (
    <div className="flex-col gap-6" style={{ display: 'flex' }}>
      <div>
        <h1 className="text-xl font-bold">Dashboard</h1>
        <p className="text-muted text-sm">Welcome back, {profile?.displayName}</p>
      </div>

      <div className="grid grid-4 gap-4">
        <StatCard label="CPU Cores" value={nodeInfo?.cpuCores?.toString() || '-'} />
        <StatCard label="RAM" value={nodeInfo?.ramGb ? `${nodeInfo.ramGb.toFixed(1)} GB` : '-'} />
        <StatCard label="Platform" value={nodeInfo?.platform || '-'} />
        <StatCard label="Uptime" value={nodeInfo?.uptimeSecs ? `${Math.floor(nodeInfo.uptimeSecs / 60)}m` : '-'} />
      </div>

      <div className="grid grid-2 gap-4">
        <div className="card">
          <h2 className="font-bold mb-4">Your Profile</h2>
          <div className="flex-col gap-2 text-sm" style={{ display: 'flex' }}>
            <Row label="Peer ID" value={profile?.peerId} />
            <Row label="Display Name" value={profile?.displayName} />
            <Row label="Followers" value={profile?.followerCount?.toString()} />
            <Row label="Following" value={profile?.followingCount?.toString()} />
            <Row label="Posts" value={profile?.postCount?.toString()} />
          </div>
        </div>

        <div className="card">
          <h2 className="font-bold mb-4">Quick Actions</h2>
          <div className="flex-col gap-2" style={{ display: 'flex' }}>
            <button onClick={() => window.location.href = '/feed'}>View Feed</button>
            <button onClick={() => window.location.href = '/network'}>Browse Network</button>
            <button onClick={() => window.location.href = '/profile'}>Edit Profile</button>
          </div>
        </div>
      </div>
    </div>
  )
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="card text-center">
      <p className="text-xl font-bold">{value}</p>
      <p className="text-muted text-sm">{label}</p>
    </div>
  )
}

function Row({ label, value }: { label: string; value?: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-muted">{label}</span>
      <span style={{
        maxWidth: 200,
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}>{value || '-'}</span>
    </div>
  )
}
