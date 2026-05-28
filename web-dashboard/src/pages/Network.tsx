import { useEffect, useState } from 'react'
import { SEARCH_PROFILES } from '../graphql/queries'
import { graphqlRequest } from '../lib/graphql-client'

interface NetworkNode {
  peerId: string
  displayName: string
  bio?: string | null
  followerCount: number
}

export default function Network() {
  const [nodes, setNodes] = useState<NetworkNode[]>([])

  useEffect(() => {
    graphqlRequest(SEARCH_PROFILES, { query: '' })
      .then(data => setNodes(data.searchProfiles || []))
      .catch(() => {})
  }, [])

  return (
    <div className="flex-col gap-6" style={{ display: 'flex' }}>
      <h1 className="text-xl font-bold">Network</h1>
      <p className="text-muted text-sm">
        Discover nodes and users connected to the Flovenet network.
      </p>

      <div className="grid grid-2 gap-4">
        {nodes.map(node => (
          <div key={node.peerId} className="card">
            <p className="font-bold">{node.displayName}</p>
            <p className="text-muted text-sm" style={{ fontFamily: 'monospace', fontSize: 12 }}>
              {node.peerId}
            </p>
            {node.bio && <p className="text-sm mt-2">{node.bio}</p>}
            <div className="flex gap-4 mt-4 text-sm text-muted">
              <span>{node.followerCount} followers</span>
            </div>
          </div>
        ))}
      </div>

      {nodes.length === 0 && (
        <p className="text-muted text-center">No nodes found on the network yet.</p>
      )}
    </div>
  )
}
