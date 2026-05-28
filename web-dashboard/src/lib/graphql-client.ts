import { getToken } from './auth'

const BASE = '/graphql'

export async function graphqlRequest(query: string, variables: Record<string, unknown> = {}) {
  const token = getToken()
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (token) headers['Authorization'] = `Bearer ${token}`

  const res = await fetch(BASE, {
    method: 'POST',
    headers,
    body: JSON.stringify({ query, variables }),
  })

  const json = await res.json()
  if (json.errors) throw new Error(json.errors[0]?.message || 'GraphQL error')
  return json.data
}
