import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetPlannedMaintenancesResp } from './GetPlannedMaintenancesResp'
import { PlannedMaintenancesResult } from './PlannedMaintenancesResult'

export async function getPlannedMaintenances (): Promise<GetPlannedMaintenancesResp | ApiErrorResp> {
  const response = await fetch('/admin/api/planned-maintenances', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as PlannedMaintenancesResult
  return GetPlannedMaintenancesResp.create(result)
}
