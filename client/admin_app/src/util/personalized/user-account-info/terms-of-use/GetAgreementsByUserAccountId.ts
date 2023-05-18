import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetAgreementsByUserAccountIdResp } from './GetAgreementsByUserAccountIdResp'
import { AgreementsResult } from './AgreementsResult'

export async function getAgreementsByUserAccountId (userAccountId: string): Promise<GetAgreementsByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/agreements-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const agreementsResult = await response.json() as AgreementsResult
  return GetAgreementsByUserAccountIdResp.create(agreementsResult)
}
