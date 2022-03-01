import { ApiErrorResp, ApiError } from '../ApiError'
import { CreateAccountResp } from './CreateAccountResp'

export async function createAccount (tempAccountId: string): Promise<CreateAccountResp | ApiErrorResp> {
  const data = { 'temp-account-id': tempAccountId }
  const response = await fetch('/api/accounts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return CreateAccountResp.create()
}
