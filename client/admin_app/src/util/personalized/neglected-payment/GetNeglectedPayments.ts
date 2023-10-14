import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { NeglectedPayment } from '../NeglectedPayment'
import { GetNeglectedPaymentsResp } from './GetNeglectedPaymentsResp'

export async function getNeglectedPayments (page: number, perPage: number): Promise<GetNeglectedPaymentsResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/neglected-payments?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { neglected_payments: NeglectedPayment[] }
  return GetNeglectedPaymentsResp.create(result.neglected_payments)
}
