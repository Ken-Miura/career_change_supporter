import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostSetMaintenanceReqResp } from './PostSetMaintenanceReqResp'
import { SetMaintenanceReq } from './SetMaintenanceReq'

export async function postSetMaintenanceReq (req: SetMaintenanceReq): Promise<PostSetMaintenanceReqResp | ApiErrorResp> {
  const response = await fetch('/admin/api/set-maintenance-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(req)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostSetMaintenanceReqResp.create()
}
