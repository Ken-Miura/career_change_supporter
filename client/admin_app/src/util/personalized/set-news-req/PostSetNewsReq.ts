import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostSetNewsReqResp } from './PostSetNewsReqResp'
import { SetNewsReq } from './SetNewsReq'

export async function postSetNewsReq (req: SetNewsReq): Promise<PostSetNewsReqResp | ApiErrorResp> {
  const response = await fetch('/admin/api/set-news-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(req)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostSetNewsReqResp.create()
}
