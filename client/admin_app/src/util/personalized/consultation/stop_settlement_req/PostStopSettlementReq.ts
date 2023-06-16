import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostStopSettlementReqResp } from './PostStopSettlementReqResp'

export async function postStopSettlementReq (settlementId: number): Promise<PostStopSettlementReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { settlement_id: settlementId }
  const response = await fetch('/admin/api/stop-settlement-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostStopSettlementReqResp.create()
}
