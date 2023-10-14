export class PostSetMaintenanceReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostSetMaintenanceReqResp {
    return new PostSetMaintenanceReqResp()
  }
}
