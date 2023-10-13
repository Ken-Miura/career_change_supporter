import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetBankAccountByUserAccountIdResp } from './GetBankAccountByUserAccountIdResp'
import { BankAccount } from './BankAccount'

export async function getBankAccountByUserAccountId (userAccountId: string): Promise<GetBankAccountByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/bank-account-id-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { bank_account: BankAccount | null }
  return GetBankAccountByUserAccountIdResp.create(result.bank_account)
}
