import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostDeleteNewsReqResp } from './PostDeleteNewsReqResp'

export async function postDeleteNewsReq (newsId: number): Promise<PostDeleteNewsReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { news_id: newsId }
  const response = await fetch('/admin/api/delete-news-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostDeleteNewsReqResp.create()
}
