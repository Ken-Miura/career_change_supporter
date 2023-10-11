import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ReceiptOfConsultation } from './ReceiptOfConsultation'
import { GetReceiptsOfConsultationResp } from './GetReceiptsOfConsultationResp'

export async function getReceiptsOfConsultation (page: number, perPage: number): Promise<GetReceiptsOfConsultationResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/receipts-of-consultation?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { receipts_of_consultation: ReceiptOfConsultation[] }
  return GetReceiptsOfConsultationResp.create(result.receipts_of_consultation)
}
