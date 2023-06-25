import { PlannedMaintenancesResult } from './PlannedMaintenancesResult'

export class GetPlannedMaintenancesResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly plannedMaintenancesResult: PlannedMaintenancesResult) {}

  public static create (plannedMaintenancesResult: PlannedMaintenancesResult): GetPlannedMaintenancesResp {
    return new GetPlannedMaintenancesResp(plannedMaintenancesResult)
  }

  public getPlannedMaintenancesResult (): PlannedMaintenancesResult {
    return this.plannedMaintenancesResult
  }
}
