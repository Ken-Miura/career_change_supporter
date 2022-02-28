import { ApiErrorResp, ApiError } from '../ApiError'
import { CreatePwdChangeReqResp } from './CreatePwdChangeReqResp'

export async function createPwdChangeReq (emailAddress: string): Promise<CreatePwdChangeReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { email_address: emailAddress }
  const response = await fetch('/api/password-change-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return CreatePwdChangeReqResp.create()
}
