import { MaintenanceTime } from './MaintenanceTime'

export type SetMaintenanceReq = {
  /* eslint-disable camelcase */
  start_time_in_jst: MaintenanceTime,
  end_time_in_jst: MaintenanceTime
  /* eslint-enable camelcase */
}
