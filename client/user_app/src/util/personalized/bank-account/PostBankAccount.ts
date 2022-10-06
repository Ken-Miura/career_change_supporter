import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { BankAccountRegisterReq } from './BankAccountRegisterReq'
import { PostBankAccountResp } from './PostBankAccountResp'

export async function postBankAccount (bankAccountRegisterReq: BankAccountRegisterReq): Promise<PostBankAccountResp | ApiErrorResp> {
  const response = await fetch('/api/bank-account', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(bankAccountRegisterReq)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostBankAccountResp.create()
}
