import { ApiErrorResp, ApiError } from '../ApiError'
import { UpdatePasswordResp } from './UpdatePasswordResp'

export async function updatePassword (pwdChangeReqId: string, password: string): Promise<UpdatePasswordResp | ApiErrorResp> {
  const data = { 'pwd-change-req-id': pwdChangeReqId, password: password }
  const response = await fetch('/api/password-update', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return UpdatePasswordResp.create()
}
