import { useEffect, useState } from 'react'
import { graphqlRequest } from '../lib/graphql-client'
import { GET_FEED, CREATE_POST } from '../graphql/queries'

interface FeedItem {
  post: { cid: string; content: string; media?: string | null; timestamp: string }
  author: { peerId: string; displayName: string }
  score?: number | null
}

export default function Feed() {
  const [items, setItems] = useState<FeedItem[]>([])
  const [newContent, setNewContent] = useState('')
  const [loading, setLoading] = useState(true)

  const loadFeed = async () => {
    try {
      const data = await graphqlRequest(GET_FEED, { limit: 50, offset: 0 })
      setItems(data.feed || [])
    } catch {
      // silent
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { loadFeed() }, [])

  const handlePost = async () => {
    if (!newContent.trim()) return
    try {
      await graphqlRequest(CREATE_POST, { content: newContent })
      setNewContent('')
      await loadFeed()
    } catch {
      // silent
    }
  }

  return (
    <div className="flex-col gap-6" style={{ display: 'flex' }}>
      <h1 className="text-xl font-bold">Feed</h1>

      <div className="card">
        <textarea
          placeholder="What's happening in the network?"
          value={newContent}
          onChange={e => setNewContent(e.target.value)}
          rows={3}
          style={{
            width: '100%',
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 'var(--radius)',
            color: 'var(--text)',
            padding: 12,
            fontFamily: 'inherit',
            fontSize: 14,
            resize: 'vertical',
          }}
        />
        <div className="flex justify-between items-center mt-4">
          <span className="text-muted text-sm">{newContent.length}/280</span>
          <button
            onClick={handlePost}
            disabled={!newContent.trim() || newContent.length > 280}
          >
            Post
          </button>
        </div>
      </div>

      {loading ? (
        <p className="text-muted text-center">Loading feed...</p>
      ) : items.length === 0 ? (
        <p className="text-muted text-center">No posts yet. Be the first!</p>
      ) : (
        items.map(item => (
          <div key={item.post.cid} className="card">
            <div className="flex items-center justify-between mb-2">
              <p className="font-bold text-sm">{item.author.displayName}</p>
              <p className="text-muted text-sm">
                {new Date(item.post.timestamp).toLocaleString()}
              </p>
            </div>
            <p style={{ lineHeight: 1.5 }}>{item.post.content}</p>
            {item.post.media && (
              <p className="text-sm mt-2">
                <a href={item.post.media} target="_blank" rel="noopener noreferrer">Media</a>
              </p>
            )}
          </div>
        ))
      )}
    </div>
  )
}
