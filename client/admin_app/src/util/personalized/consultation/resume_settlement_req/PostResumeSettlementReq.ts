import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostResumeSettlementReqResp } from './PostResumeSettlementReqResp'

export async function postResumeSettlementReq (stoppedSettlementId: number): Promise<PostResumeSettlementReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { stopped_settlement_id: stoppedSettlementId }
  const response = await fetch('/admin/api/resume-settlement-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostResumeSettlementReqResp.create()
}
