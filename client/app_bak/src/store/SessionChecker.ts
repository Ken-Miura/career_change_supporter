// Copyright 2021 Ken Miura

export type SessionState = 'none' | 'active'

export async function getSessionState (): Promise<SessionState> {
  let response
  try {
    response = await fetch('/api/user/session-state', {
      method: 'GET'
    })
  } catch (e) {
    console.log(`failed to get response: ${e}`)
    return 'none'
  }
  if (!response.ok) {
    const text = await response.text()
    console.log('failed to get session state. status code: %d, body: %s', response.status, text)
    return 'none'
  }
  return 'active'
}
