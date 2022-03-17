import { ApiErrorResp, ApiError } from '../ApiError'
import { CreateTempAccountResp } from './CreateTempAccountResp'

export async function createTempAccount (emailAddress: string, password: string): Promise<CreateTempAccountResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { email_address: emailAddress, password: password }
  const response = await fetch('/api/temp-accounts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return CreateTempAccountResp.create()
}
