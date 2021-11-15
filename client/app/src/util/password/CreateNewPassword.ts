import { ApiErrorResp, ApiError } from '../ApiError'
import { CreateNewPasswordResp } from './CreateNewPasswordResp'

export async function createNewPassword (emailAddress: string, password: string): Promise<CreateNewPasswordResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { email_address: emailAddress, password: password }
  const response = await fetch('/api/new-password', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as { email_address: string }
  return CreateNewPasswordResp.create(result.email_address)
}
