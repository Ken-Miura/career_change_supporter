import { ApiErrorResp, ApiError } from '../ApiError'
import { CreateAccountResp } from './CreateAccountResp'

export async function createAccount (jsonData: string): Promise<CreateAccountResp | ApiErrorResp> {
  const response = await fetch('/api/accounts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: jsonData
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return CreateAccountResp.create()
}
