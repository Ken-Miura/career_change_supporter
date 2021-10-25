export type RefreshResult = 'SUCCESS' | 'FAILURE'

export async function refresh (): Promise<RefreshResult> {
  const response = await fetch('/api/refresh', {
    method: 'GET'
  })
  if (!response.ok) {
    return 'FAILURE'
  }
  return 'SUCCESS'
}
