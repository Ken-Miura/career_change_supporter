import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { User } from './User'
import { GetUsersByDateOfBirthResp } from './GetUsersByDateOfBirthResp'

export async function getUsersByDateOfBirth (year: number, month: number, day: number): Promise<GetUsersByDateOfBirthResp | ApiErrorResp> {
  const params = { year: year.toString(), month: month.toString(), day: day.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/users-by-date-of-birth?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const users = await response.json() as User[]
  return GetUsersByDateOfBirthResp.create(users)
}
