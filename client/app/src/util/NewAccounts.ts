import { ApiErrorResp, ApiError } from './ApiError'

export class CreateTempAccountResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly emailAddress: string) {}
  public static create (emailAddress: string): CreateTempAccountResp {
    return new CreateTempAccountResp(emailAddress)
  }

  public getEmailAddress (): string {
    return this.emailAddress
  }
}

export async function createTempAccount (emailAddress: string, password: string): Promise<CreateTempAccountResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { email_address: emailAddress, password: password }
  const response = await fetch('/api/temp-accounts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as { email_address: string }
  return CreateTempAccountResp.create(result.email_address)
}
