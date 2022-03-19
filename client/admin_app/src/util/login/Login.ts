import { ApiErrorResp, ApiError } from '../ApiError'
import { LoginResp } from './LoginResp'

export async function login (emailAddress: string, password: string): Promise<LoginResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { email_address: emailAddress, password: password }
  const response = await fetch('/admin/api/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return LoginResp.create()
}
