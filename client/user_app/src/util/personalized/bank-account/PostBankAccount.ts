import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { BankAccount } from '../BankAccount'
import { PostBankAccountResp } from './PostBankAccountResp'

export async function postBankAccount (bankAccount: BankAccount): Promise<PostBankAccountResp | ApiErrorResp> {
  const response = await fetch('/api/bank-account', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(bankAccount)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostBankAccountResp.create()
}
