import { SetMaintenanceReqResult } from './SetMaintenanceReqResult'

export class PostSetMaintenanceReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly setMaintenanceReqResult: SetMaintenanceReqResult) {}

  public static create (setMaintenanceReqResult: SetMaintenanceReqResult): PostSetMaintenanceReqResp {
    return new PostSetMaintenanceReqResp(setMaintenanceReqResult)
  }

  public getSetMaintenanceReqResult (): SetMaintenanceReqResult {
    return this.setMaintenanceReqResult
  }
}
